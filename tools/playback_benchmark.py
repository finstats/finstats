#!/usr/bin/env python3
"""Measure real DirectPlay on disposable Jellyfin and FinStats Docker instances.

Requires Python 3.9+, Linux cgroup v2, Docker, local ffmpeg and explicitly cached
images. Creates an internal network, scratch data, generated media and loopback
ports. Never accepts an existing Jellyfin URL or existing tracker container.
"""
import argparse
from contextlib import closing, contextmanager
import hashlib
import http.client
import http.server
import ipaddress
import json
import math
import os
from pathlib import Path
import re
import secrets
import signal
import shutil
import sqlite3
import socket
import subprocess
import sys
import tempfile
import time
import threading
import urllib.error
import urllib.parse
import urllib.request

import idle_benchmark as idle

LABEL = 'no.finstats.playback-benchmark'


class PlaybackError(Exception):
    pass


def parse_args(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--jellyfin-image', required=True, help='explicit locally cached Jellyfin image reference')
    parser.add_argument('--finstats-image', action='append', required=True, metavar='LABEL=IMAGE', help='repeat for local FinStats versions; cached images only')
    parser.add_argument('--output', type=Path, required=True, help='new private results directory')
    parser.add_argument('--duration', type=float, default=300)
    parser.add_argument('--interval', type=float, default=10)
    parser.add_argument('--warmup', type=float, default=30, help='settle after library readiness, seconds')
    parser.add_argument('--timeout', type=float, default=5, help='individual host command and HTTP timeout')
    parser.add_argument('--startup-timeout', type=float, default=180, help='startup and library readiness deadline')
    parser.add_argument('--streams', type=int, default=1, choices=range(1, 5))
    parser.add_argument('--progress-every', type=float, default=5, help='client progress reporting interval')
    parser.add_argument('--ffmpeg', default='ffmpeg', help='local ffmpeg executable')
    args = parser.parse_args(argv)
    values = (args.duration, args.interval, args.timeout, args.startup_timeout, args.progress_every)
    if any(not math.isfinite(x) or x <= 0 for x in values) or not math.isfinite(args.warmup) or args.warmup < 0:
        parser.error('times must be finite and positive (warmup may be zero)')
    ratio = args.duration / args.interval
    if not math.isfinite(ratio) or not math.isclose(ratio, round(ratio), abs_tol=1e-9) or not 6 <= ratio <= 100000:
        parser.error('duration must be an exact multiple of interval with at least six intervals for play/pause/resume/stop')
    images = []
    for value in args.finstats_image:
        name, sep, image = value.partition('=')
        if not sep or not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_. -]{0,79}', name):
            parser.error('FinStats images require a plain LABEL=IMAGE')
        images.append((name, image))
    if len(set(name for name, _ in images)) != len(images):
        parser.error('FinStats labels must be unique')
    for image in [args.jellyfin_image] + [image for _, image in images]:
        if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_./:@-]{0,254}', image):
            parser.error('image references must contain no whitespace, options or shell text')
    args.finstats_image = images
    return args


def docker_command(argv, timeout=30):
    try:
        return idle.host_command(['docker', *argv], timeout, 'Docker operation').strip()
    except idle.SampleError as exc:
        raise PlaybackError(str(exc)) from None


class LoopbackProxy:
    """HTTP only, to one newly created internal-network container, never an open proxy."""
    def __init__(self, address, port, timeout):
        try:
            if not ipaddress.ip_address(address).is_private:
                raise ValueError()
        except ValueError:
            raise PlaybackError('created container has no private numeric bridge address') from None
        self.connections = set()
        self.lock = threading.Lock()
        owner = self
        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass
            def do_GET(self): self.forward()
            def do_HEAD(self): self.forward()
            def do_POST(self): self.forward()
            def forward(self):
                upstream = None
                with owner.lock:
                    owner.connections.add(self.connection)
                try:
                    length = int(self.headers.get('Content-Length', '0'))
                    if not 0 <= length <= 8 * 1024 * 1024 or not self.path.startswith('/') or self.path.startswith('//'):
                        self.send_error(400)
                        return
                    data = self.rfile.read(length) if length else None
                    excluded = {'host', 'connection', 'transfer-encoding', 'keep-alive', 'upgrade', 'proxy-authorization'}
                    headers = {key: value for key, value in self.headers.items() if key.lower() not in excluded}
                    upstream = http.client.HTTPConnection(address, port, timeout=max(60, timeout))
                    upstream.request(self.command, self.path, body=data, headers=headers)
                    response = upstream.getresponse()
                    self.send_response(response.status)
                    for key, value in response.getheaders():
                        if key.lower() not in excluded:
                            self.send_header(key, value)
                    self.send_header('Connection', 'close')
                    self.end_headers()
                    if self.command != 'HEAD':
                        while True:
                            chunk = response.read(65536)
                            if not chunk:
                                break
                            self.wfile.write(chunk)
                except (OSError, ValueError, http.client.HTTPException):
                    self.close_connection = True
                finally:
                    if upstream:
                        upstream.close()
                    with owner.lock:
                        owner.connections.discard(self.connection)
        self.server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        self.server.daemon_threads = True
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.url = 'http://127.0.0.1:' + str(self.server.server_port)

    def close(self):
        self.server.shutdown()
        with self.lock:
            for connection in self.connections:
                try:
                    connection.shutdown(socket.SHUT_RDWR)
                    connection.close()
                except OSError:
                    pass
        self.server.server_close()
        self.thread.join(timeout=2)


class Sandbox:
    def __init__(self, root, run_id, docker=docker_command, manifest=None):
        self.root = root.resolve()
        self.run_id = run_id
        self.docker = docker
        self.network_id = None
        self.containers = []
        self.cleanup_errors = []
        self.proxies = []
        self.manifest = manifest

    def save(self):
        if self.manifest:
            data = {'run_id': self.run_id, 'label': LABEL, 'network_id': self.network_id, 'containers': self.containers}
            self.manifest.seek(0)
            self.manifest.truncate()
            self.manifest.write(json.dumps(data, indent=2) + '\n')
            self.manifest.flush()

    def network(self):
        self.network_id = self.docker(['network', 'create', '--internal', '--label', LABEL + '=' + self.run_id, self.run_id])
        self.save()
        return self.network_id

    def container(self, role, image, mounts, env, port):
        name = self.run_id + '-' + role
        command = ['create', '--pull=never', '--name', name, '--label', LABEL + '=' + self.run_id,
                   '--network', self.network_id or self.run_id, '--user', str(os.getuid()) + ':' + str(os.getgid()),
                   '--cpus', '2', '--memory', '1g', '--pids-limit', '256']
        for source, (destination, readonly) in mounts.items():
            source = source.resolve()
            if self.root != source and self.root not in source.parents:
                raise PlaybackError('refusing a mount outside this run scratch directory')
            source.mkdir(parents=True, exist_ok=True)
            command.extend(['--mount', 'type=bind,source=' + str(source) + ',target=' + destination + (',readonly' if readonly else '')])
        env_file = None
        if env:
            env_file = self.root / ('.env-' + role)
            with idle.private_file(env_file) as handle:
                for key, value in env.items():
                    if any(char in key + value for char in ('\r', '\n', '\x00')):
                        raise PlaybackError('invalid environment value for disposable container')
                    handle.write(key + '=' + value + '\n')
            command.extend(['--env-file', str(env_file)])
        command.append(image)
        try:
            identity = self.docker(command)
        finally:
            if env_file:
                env_file.unlink()
        if not re.fullmatch(r'[A-Za-z0-9-]+', identity):
            raise PlaybackError('Docker returned an invalid container identity')
        self.containers.append({'id': identity, 'name': name, 'role': role, 'image': image})
        self.save()
        self.docker(['start', identity])
        return identity, name

    def proxy_url(self, container, port, timeout):
        if container not in [item['id'] for item in self.containers]:
            raise PlaybackError('refusing a proxy to a container outside this run')
        address = self.docker(['inspect', '--format', '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}', container])
        proxy = LoopbackProxy(address, port, timeout)
        self.proxies.append(proxy)
        return proxy.url

    def cleanup(self):
        for proxy in self.proxies:
            proxy.close()
        for container in reversed(self.containers):
            try:
                label = self.docker(['inspect', '--format', '{{index .Config.Labels "' + LABEL + '"}}', container['id']])
                if label != self.run_id:
                    raise PlaybackError('container ownership label changed')
                self.docker(['rm', '-f', container['id']])
            except PlaybackError:
                self.cleanup_errors.append('could not remove owned container ' + container['id'])
        if self.network_id:
            try:
                label = self.docker(['network', 'inspect', '--format', '{{index .Labels "' + LABEL + '"}}', self.network_id])
                if label != self.run_id:
                    raise PlaybackError('network ownership label changed')
                self.docker(['network', 'rm', self.network_id])
            except PlaybackError:
                self.cleanup_errors.append('could not remove owned network ' + self.network_id)


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file, code, message, headers, newurl):
        raise PlaybackError('disposable Jellyfin unexpectedly redirected a request')


class Jellyfin:
    def __init__(self, url, device_id, timeout=5):
        parsed = urllib.parse.urlsplit(url)
        if parsed.scheme != 'http' or parsed.hostname != '127.0.0.1' or not parsed.port or parsed.path or parsed.query or parsed.username:
            raise PlaybackError('playback API must use a created loopback HTTP port')
        self.url = url
        self.device_id = device_id
        self.timeout = timeout
        self.token = None
        self.user_id = None
        self.opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())

    def headers(self):
        headers = {'Authorization': 'MediaBrowser Client="FinStats Benchmark", Device="Generated media", DeviceId="' + self.device_id + '", Version="1.0"',
                   'Accept': 'application/json; profile="PascalCase"'}
        if self.token:
            headers['Authorization'] += ', Token="' + self.token + '"'
        return headers

    def request(self, method, path, body=None):
        if not path.startswith('/') or path.startswith('//') or '\r' in path or '\n' in path:
            raise PlaybackError('invalid disposable Jellyfin API path')
        data = json.dumps(body).encode() if body is not None else None
        headers = self.headers()
        if data:
            headers['Content-Type'] = 'application/json'
        request = urllib.request.Request(self.url + path, data=data, headers=headers, method=method)
        try:
            with self.opener.open(request, timeout=self.timeout) as response:
                payload = response.read(8 * 1024 * 1024 + 1)
                if len(payload) > 8 * 1024 * 1024:
                    raise PlaybackError('disposable Jellyfin response exceeded limit')
                return json.loads(payload) if payload else None
        except urllib.error.HTTPError as exc:
            raise PlaybackError('disposable Jellyfin returned HTTP ' + str(exc.code) + ' for ' + method + ' ' + path.split('?')[0]) from None
        except (urllib.error.URLError, OSError, ValueError):
            raise PlaybackError('disposable Jellyfin did not return a readable response') from None

    def authenticate(self, password):
        data = object_response(self.request('POST', '/Users/AuthenticateByName', {'Username': 'benchmark', 'Pw': password}), 'authentication')
        user = object_response(data.get('User'), 'authentication user')
        if not isinstance(data.get('AccessToken'), str) or not data['AccessToken'] or not isinstance(user.get('Id'), str) or not user['Id']:
            raise PlaybackError('disposable Jellyfin returned incomplete authentication data')
        self.token = data['AccessToken']
        self.user_id = user['Id']

    def playback_info(self, item_id):
        profile = {'Name': 'Benchmark DirectPlay', 'MaxStreamingBitrate': 20000000,
                   'DirectPlayProfiles': [{'Container': 'mp4,mov', 'Type': 'Video', 'VideoCodec': 'h264', 'AudioCodec': 'aac'}],
                   'TranscodingProfiles': []}
        result = self.request('POST', '/Items/' + item_id + '/PlaybackInfo', {
            'UserId': self.user_id, 'IsPlayback': True, 'EnableDirectPlay': True, 'EnableDirectStream': False,
            'EnableTranscoding': False, 'DeviceProfile': profile})
        result = object_response(result, 'playback info')
        sources = result.get('MediaSources')
        if not isinstance(sources, list):
            raise PlaybackError('disposable Jellyfin returned invalid playback sources')
        source = next((s for s in sources if isinstance(s, dict) and s.get('SupportsDirectPlay') and s.get('Container') in ('mp4', 'mov') and isinstance(s.get('Id'), str) and s['Id']), None)
        if not source or not isinstance(result.get('PlaySessionId'), str) or not result['PlaySessionId']:
            raise PlaybackError('generated movie is not available as verified DirectPlay')
        return source['Id'], result['PlaySessionId']

    def report(self, kind, item_id, source, session, ticks, paused):
        suffix = {'start': '', 'progress': '/Progress', 'stop': '/Stopped'}[kind]
        body = {'ItemId': item_id, 'MediaSourceId': source, 'PlaySessionId': session, 'PositionTicks': int(ticks),
                'IsPaused': paused, 'CanSeek': True, 'PlayMethod': 'DirectPlay', 'VolumeLevel': 100,
                'AudioStreamIndex': 1, 'SubtitleStreamIndex': -1}
        self.request('POST', '/Sessions/Playing' + suffix, body)

    def stream_url(self, item_id, source, session):
        query = urllib.parse.urlencode({'Static': 'true', 'MediaSourceId': source, 'PlaySessionId': session, 'DeviceId': self.device_id})
        return self.url + '/Videos/' + item_id + '/stream.mp4?' + query


def wait_until(check, timeout, message):
    deadline = time.monotonic() + timeout
    while True:
        try:
            result = check()
            if result:
                return result
        except PlaybackError:
            pass
        if time.monotonic() >= deadline:
            raise PlaybackError(message)
        time.sleep(0.5)


def object_response(value, context):
    if not isinstance(value, dict):
        raise PlaybackError('disposable Jellyfin returned no ' + context + ' object')
    return value


def public_info(api):
    info = api.request('GET', '/System/Info/Public')
    if not isinstance(info, dict) or not all(isinstance(info.get(key), str) and info[key] for key in ('Id', 'Version')) or type(info.get('StartupWizardCompleted')) is not bool:
        return None
    return info


def bootstrap(api, password, startup_timeout):
    server = wait_until(lambda: public_info(api), startup_timeout, 'disposable Jellyfin startup identity did not become ready')
    if server['StartupWizardCompleted'] is not False:
        raise PlaybackError('refusing an already configured Jellyfin instance')
    wait_until(lambda: api.request('GET', '/Startup/User'), startup_timeout, 'disposable Jellyfin setup API did not become ready')
    api.request('POST', '/Startup/Configuration', {'ServerName': 'FinStats Benchmark', 'UICulture': 'en-US',
        'MetadataCountryCode': 'US', 'PreferredMetadataLanguage': 'en'})
    api.request('POST', '/Startup/User', {'Name': 'benchmark', 'Password': password})
    api.request('POST', '/Startup/RemoteAccess', {'EnableRemoteAccess': True, 'EnableAutomaticPortMapping': False})
    api.request('POST', '/Startup/Complete')
    api.authenticate(password)
    options = {'EnableRealtimeMonitor': False, 'EnableInternetProviders': False,
               'PathInfos': [{'Path': '/media'}], 'EnableChapterImageExtraction': False,
               'TypeOptions': [{'Type': 'Movie', 'MetadataFetchers': [], 'ImageFetchers': [],
                                'MetadataFetcherOrder': [], 'ImageFetcherOrder': []}]}
    api.request('POST', '/Library/VirtualFolders?name=Benchmark&collectionType=movies&refreshLibrary=true', {'LibraryOptions': options})
    def scan_idle():
        tasks = api.request('GET', '/ScheduledTasks')
        if not isinstance(tasks, list):
            raise PlaybackError('disposable Jellyfin returned no task list')
        scans = [task for task in tasks if isinstance(task, dict) and task.get('Key') == 'RefreshLibrary']
        return bool(scans) and all(task.get('State') == 'Idle' for task in scans)
    # A refresh queued while the startup scan runs may be coalesced with that
    # older scan, whose library list predates our movie. Start a scan after idle.
    wait_until(scan_idle, startup_timeout, 'disposable Jellyfin initial library scan did not settle')
    api.request('POST', '/Library/Refresh')
    def movie():
        answer = api.request('GET', '/Items?' + urllib.parse.urlencode({'Recursive': 'true', 'IncludeItemTypes': 'Movie', 'UserId': api.user_id, 'Fields': 'MediaSources'}))
        answer = object_response(answer, 'library items')
        items = answer.get('Items')
        if not isinstance(items, list):
            raise PlaybackError('disposable Jellyfin returned invalid library items')
        return next((item for item in items if isinstance(item, dict) and isinstance(item.get('RunTimeTicks'), (int, float)) and item['RunTimeTicks'] > 0), None)
    item = wait_until(movie, startup_timeout, 'generated movie did not become ready')
    server = wait_until(lambda: info if (info := public_info(api)) and info['StartupWizardCompleted'] is True else None, startup_timeout, 'disposable Jellyfin final identity did not become ready')
    return item, server


def generate_media(ffmpeg, destination, duration):
    command = [ffmpeg, '-hide_banner', '-loglevel', 'error', '-nostdin', '-f', 'lavfi', '-i',
               'testsrc2=size=320x180:rate=12', '-f', 'lavfi', '-i', 'sine=frequency=440:sample_rate=48000',
               '-t', str(duration), '-c:v', 'libx264', '-preset', 'ultrafast', '-threads', '1',
               '-pix_fmt', 'yuv420p', '-c:a', 'aac', '-b:a', '64k', '-movflags', '+faststart', str(destination)]
    try:
        result = subprocess.run(command, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                timeout=max(60, duration), check=False)
    except (OSError, subprocess.TimeoutExpired):
        raise PlaybackError('generated-media encoding failed or timed out') from None
    if result.returncode:
        raise PlaybackError('ffmpeg could not encode the generated H.264/AAC fixture')


class MediaClient:
    def __init__(self, ffmpeg, api, movie, source, session, duration, timeout):
        self.ffmpeg, self.api = ffmpeg, api
        self.movie, self.source, self.session = movie, source, session
        self.duration, self.timeout = duration, timeout
        self.process = None
        self.reader = None
        self.lock = threading.Lock()
        self.position_ticks = 0
        self.frames = 0
        self.last_progress = time.monotonic()
        self.paused = False

    def snapshot(self):
        with self.lock:
            return {'position_ticks': self.position_ticks, 'frames': self.frames,
                    'decoded_seconds': self.position_ticks / 10000000}

    def start(self):
        headers = ''.join(key + ': ' + value + '\r\n' for key, value in self.api.headers().items())
        command = [self.ffmpeg, '-hide_banner', '-loglevel', 'error', '-nostdin', '-re', '-rw_timeout',
                   str(int(self.timeout * 1000000)), '-headers', headers, '-i',
                   self.api.stream_url(self.movie, self.source, self.session), '-t', str(self.duration),
                   '-map', '0:v:0', '-map', '0:a:0?', '-threads', '1', '-stats_period', '0.5',
                   '-progress', 'pipe:1', '-f', 'null', '-']
        env = {key: value for key, value in os.environ.items() if key.lower() not in ('http_proxy', 'https_proxy', 'all_proxy')}
        try:
            self.process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                                            text=True, start_new_session=True, env=env)
        except OSError:
            raise PlaybackError('could not start local ffmpeg media client') from None
        def read_progress():
            for line in self.process.stdout:
                key, sep, value = line.strip().partition('=')
                if sep and value.isdigit():
                    with self.lock:
                        if key == 'out_time_us':
                            self.position_ticks = int(value) * 10
                            self.last_progress = time.monotonic()
                        elif key == 'frame':
                            self.frames = int(value)
        self.reader = threading.Thread(target=read_progress, daemon=True)
        self.reader.start()
        wait_until(lambda: self.snapshot()['frames'] > 0, max(15, self.timeout * 2), 'ffmpeg did not decode generated HTTP media')

    def pause(self):
        if self.process and self.process.poll() is None:
            os.killpg(self.process.pid, signal.SIGSTOP)
            self.paused = True

    def resume(self):
        if self.process and self.process.poll() is None:
            os.killpg(self.process.pid, signal.SIGCONT)
            self.paused = False
            self.last_progress = time.monotonic()

    def check(self):
        if self.process and self.process.poll() is not None:
            raise PlaybackError('ffmpeg media client exited before the scenario stop')
        if not self.paused and time.monotonic() - self.last_progress > max(15, self.timeout * 3):
            raise PlaybackError('ffmpeg media decoding stopped advancing')

    def close(self):
        if self.process:
            if self.paused:
                self.resume()
            idle.stop_process(self.process)
            if self.reader:
                self.reader.join(timeout=2)
            self.process = None


def phase(elapsed, duration, interval):
    if elapsed >= duration - 2 * interval:
        return 'stopped'
    if duration / 3 <= elapsed < duration / 3 + interval:
        return 'paused'
    return 'playing'


class Workload:
    def __init__(self, members, duration, interval, progress_every, output):
        self.members = members
        self.duration, self.interval, self.progress_every = duration, interval, progress_every
        self.output = output
        self.previous_phase = 'playing'
        self.last_report = -progress_every
        self.errors = []
        self.stop_event = threading.Event()
        self.events = []
        self.started_monotonic_s = None

    def emit(self, kind, index, elapsed, snapshot):
        event = {'type': kind, 'stream': index, 'elapsed_s': elapsed, 'utc': idle.utc(), 'monotonic_s': time.monotonic(), **snapshot}
        self.events.append(event)
        self.output.write(json.dumps(event) + '\n')
        self.output.flush()

    def step(self, elapsed):
        state = phase(elapsed, self.duration, self.interval)
        changed = state != self.previous_phase
        if self.previous_phase == 'stopped':
            return
        for index, (api, client, movie, source, session) in enumerate(self.members):
            client.check()
            if changed and state == 'paused':
                client.pause()
            elif changed and state == 'playing':
                client.resume()
            snapshot = client.snapshot()
            if state == 'stopped':
                api.report('stop', movie, source, session, snapshot['position_ticks'], False)
                client.close()
                self.emit('stop', index, elapsed, snapshot)
            elif changed or elapsed - self.last_report >= self.progress_every:
                api.report('progress', movie, source, session, snapshot['position_ticks'], state == 'paused')
                self.emit('pause' if state == 'paused' and changed else ('resume' if changed else 'progress'), index, elapsed, snapshot)
        if changed or elapsed - self.last_report >= self.progress_every:
            self.last_report = elapsed
        self.previous_phase = state

    def run(self):
        start = time.monotonic()
        self.started_monotonic_s = start
        try:
            while not self.stop_event.is_set() and self.previous_phase != 'stopped':
                self.step(time.monotonic() - start)
                self.stop_event.wait(0.2)
        except (PlaybackError, OSError):
            self.errors.append('playback client or lifecycle report failed during measurement')

    def finish(self):
        self.stop_event.set()
        if self.previous_phase != 'stopped':
            for api, client, movie, source, session in self.members:
                try:
                    api.report('stop', movie, source, session, client.snapshot()['position_ticks'], False)
                except PlaybackError:
                    self.errors.append('final playback stop report failed')
                finally:
                    client.close()
            self.previous_phase = 'stopped'


def history(data, movie=None):
    database = data / 'finstats.db'
    if not database.is_file():
        return None
    try:
        with closing(sqlite3.connect(database.as_uri() + '?mode=ro', uri=True, timeout=1)) as connection:
            if movie is None:
                return connection.execute('SELECT (SELECT COUNT(*) FROM items),(SELECT COUNT(*) FROM users)').fetchone()
            count, closed = connection.execute('SELECT COUNT(*),COALESCE(SUM(active=0),0) FROM visible_playbacks WHERE item_id=? AND source=\'live\'', (movie.replace('-', '').lower(),)).fetchone()
            events = dict(connection.execute('SELECT e.kind,COUNT(*) FROM playback_events e JOIN visible_playbacks p ON p.id=e.playback_id WHERE p.item_id=? GROUP BY e.kind', (movie.replace('-', '').lower(),)))
            return {'plays': count, 'closed_plays': closed, 'events': events}
    except sqlite3.Error:
        return None


def wait_tracker(data, startup_timeout):
    return wait_until(lambda: (counts if counts and min(counts) >= 1 else None) if (counts := history(data)) else None,
                      startup_timeout, 'throwaway FinStats library and users did not become ready')


def write_json(path, data):
    # scenario.json is opened once and rewritten only by this run.
    with open(path, 'w', encoding='utf-8', opener=lambda name, flags: os.open(name, flags, 0o600)) as handle:
        json.dump(data, handle, indent=2)
        handle.write('\n')


PLAYBACK_CAVEATS = [
    'Generated 320x180 H.264/AAC media is fetched over real Jellyfin HTTP and decoded in real time by local ffmpeg; positions come from decoder progress, not invented clocks.',
    'Start, progress, pause, resume and stop reports target only the newly created Jellyfin. Pause suspends the decoder process. The final two sampling intervals are a stopped tail, so the full-window mean includes mixed playback phases.',
    'Only tracker cgroups are included in the comparison. The disposable Jellyfin and local decoder processes are workload generators outside tracker totals; their host CPU and cache effects still influence the run.',
    'This is DirectPlay on a tiny generated library, not a transcoding benchmark, real-user traffic replay, UI load test or a prediction for a production library. All FinStats versions share the same disposable Jellyfin and run simultaneously.',
    'Existing remote trackers are not automatically sampled as playback participants. To compare other trackers they must be provisioned against this same disposable Jellyfin; independent SSH idle runs are not valid playback comparisons.',
]


def run_playback(args):
    args.output.mkdir(mode=0o700, parents=True, exist_ok=False)
    scenario = {'format_version': 1, 'status': 'starting', 'started_utc': idle.utc(), 'kind': 'generated-directplay',
                'duration_s': args.duration, 'interval_s': args.interval, 'streams': args.streams,
                'progress_every_s': args.progress_every, 'warmup_s': args.warmup,
                'pause_at_s': args.duration / 3, 'pause_for_s': args.interval,
                'stop_at_s': args.duration - 2 * args.interval, 'errors': [], 'caveats': PLAYBACK_CAVEATS}
    sandbox = None
    root = None
    members = []
    workload = None
    worker = None
    events = None
    manifest = None
    result = 1
    try:
        idle.require_local_docker(args.timeout)
        images = [args.jellyfin_image] + [image for _, image in args.finstats_image]
        scenario['images'] = [{'reference': image, 'id': docker_command(['image', 'inspect', '--format', '{{.Id}}', image], args.timeout)} for image in images]
        # Check images before allocating any runtime resources. Docker create also uses --pull=never.
        root = Path(tempfile.mkdtemp(prefix='finstats-playback-')).resolve()
        root.chmod(0o700)
        run_id = 'finstats-bench-' + secrets.token_hex(8)
        manifest = idle.private_file(args.output / 'resources.json')
        sandbox = Sandbox(root, run_id, lambda argv: docker_command(argv, max(10, args.timeout)), manifest)
        media = root / 'media'
        media.mkdir()
        movie_file = media / 'Benchmark (2026).mp4'
        generate_media(args.ffmpeg, movie_file, args.duration + 60)
        scenario['media'] = {'kind': 'testsrc2+sine', 'video': 'H.264', 'audio': 'AAC', 'width': 320, 'height': 180,
                             'fps': 12, 'duration_s': args.duration + 60, 'bytes': movie_file.stat().st_size,
                             'sha256': hashlib.sha256(movie_file.read_bytes()).hexdigest()}
        sandbox.network()
        jf_id, jf_name = sandbox.container('jellyfin', args.jellyfin_image,
            {root / 'jellyfin-config': ('/config', False), root / 'jellyfin-cache': ('/cache', False), media: ('/media', True)}, {}, 8096)
        api = Jellyfin(sandbox.proxy_url(jf_id, 8096, args.timeout), 'setup-' + secrets.token_hex(8), args.timeout)
        password = secrets.token_urlsafe(24)
        movie, server = bootstrap(api, password, args.startup_timeout)
        scenario['jellyfin'] = {'version': server.get('Version'), 'server_id': server.get('Id'), 'image': args.jellyfin_image}
        tracker_data = []
        targets = []
        for index, (label, image) in enumerate(args.finstats_image):
            data = root / ('tracker-' + str(index))
            _, name = sandbox.container('tracker-' + str(index), image, {data: ('/data', False)},
                {'JELLYFIN_URL': 'http://' + jf_name + ':8096', 'JELLYFIN_API_KEY': api.token,
                 'FINSTATS_PUBLIC_IP_URL': 'http://127.0.0.1:1', 'TZ': 'UTC'}, 8080)
            tracker_data.append((label, data))
            targets.append({'name': label, 'ssh': None, 'containers': [name], 'ssh_options': [], 'version': image,
                            'notes': 'Fresh generated library; DirectPlay workload; 2 CPU / 1 GiB container limits'})
        for _, data in tracker_data:
            wait_tracker(data, args.startup_timeout)
        time.sleep(args.warmup)
        events = idle.private_file(args.output / 'playback-events.jsonl')
        for index in range(args.streams):
            client_api = Jellyfin(api.url, 'player-' + str(index) + '-' + secrets.token_hex(8), args.timeout)
            client_api.authenticate(password)
            source, session = client_api.playback_info(movie['Id'])
            client = MediaClient(args.ffmpeg, client_api, movie['Id'], source, session, args.duration + 30, args.timeout)
            members.append((client_api, client, movie['Id'], source, session))
            client.start()
            client_api.report('start', movie['Id'], source, session, client.snapshot()['position_ticks'], False)
        workload = Workload(members, args.duration, args.interval, args.progress_every, events)
        for index, (_, client, _, _, _) in enumerate(members):
            workload.emit('start', index, 0, client.snapshot())
        scenario['status'] = 'measuring'
        write_json(args.output / 'scenario.json', scenario)
        worker = threading.Thread(target=workload.run, daemon=True)
        worker.start()
        sample_args = argparse.Namespace(output=args.output / 'resources', duration=args.duration,
                                         interval=args.interval, timeout=args.timeout)
        result = idle.run(sample_args, targets)
        workload.stop_event.set()
        worker.join(timeout=args.timeout * args.streams + 5)
        if worker.is_alive():
            raise PlaybackError('playback report worker did not stop')
        workload.finish()
        if result == 130:
            raise KeyboardInterrupt
        scenario['errors'].extend(workload.errors)
        scenario['workload_started_monotonic_s'] = workload.started_monotonic_s
        scenario['decoded'] = [client.snapshot() for _, client, _, _, _ in members]
        scenario['history'] = {}
        for label, data in tracker_data:
            observed = wait_until(lambda: value if (value := history(data, movie['Id'])) and value['closed_plays'] >= args.streams else None,
                                  max(10, args.timeout * 2), 'FinStats did not close every generated play')
            scenario['history'][label] = observed
            if any(observed['events'].get(kind, 0) < args.streams for kind in ('start', 'pause', 'resume', 'stop')):
                scenario['errors'].append('FinStats did not retain every lifecycle event for ' + label)
        expected_events = {'start', 'pause', 'resume', 'stop'}
        if not expected_events.issubset({event['type'] for event in workload.events}):
            scenario['errors'].append('playback phase sequence did not complete')
        scenario['status'] = 'complete' if result == 0 and not scenario['errors'] else 'incomplete'
        result = 0 if scenario['status'] == 'complete' else (130 if result == 130 else 1)
    except KeyboardInterrupt:
        scenario['status'] = 'interrupted'
        result = 130
    except (PlaybackError, idle.SampleError, OSError, KeyError, TypeError, ValueError) as exc:
        scenario['status'] = 'failed'
        scenario['errors'].append(str(exc) if isinstance(exc, (PlaybackError, idle.SampleError)) else 'playback setup or artifact operation failed')
        result = 1
    finally:
        if workload:
            workload.stop_event.set()
            if worker:
                worker.join(timeout=args.timeout * args.streams + 5)
            workload.finish()
        for _, client, _, _, _ in members:
            client.close()
        if events:
            events.close()
        if sandbox:
            sandbox.cleanup()
            scenario['cleanup_errors'] = sandbox.cleanup_errors
            if sandbox.cleanup_errors:
                scenario['scratch_retained'] = str(root)
                scenario['status'] = 'incomplete'
                result = 1
        if manifest:
            manifest.close()
        if root and (not sandbox or not sandbox.cleanup_errors):
            try:
                shutil.rmtree(root)
            except OSError:
                scenario.setdefault('cleanup_errors', []).append('could not remove scratch directory')
                scenario['scratch_retained'] = str(root)
                scenario['status'] = 'incomplete'
                result = 1
        scenario['finished_utc'] = idle.utc()
        write_json(args.output / 'scenario.json', scenario)
    summary_file = args.output / 'resources' / 'summary.json'
    if summary_file.exists():
        summary = json.loads(summary_file.read_text())
        summary['scenario'] = scenario
        if scenario['status'] != 'complete':
            summary['status'] = scenario['status']
        summary['methodology'] = [text.replace('Observational idle deployment', 'Observational generated DirectPlay') for text in summary['methodology']] + PLAYBACK_CAVEATS
        write_json(summary_file, summary)
        table = idle.markdown(summary).replace('# Idle resource comparison', '# Playback resource comparison').replace('Observational idle deployment', 'Observational generated DirectPlay')
        with idle.private_file(args.output / 'comparison.md') as handle:
            handle.write(table + '\n' + '\n'.join('- ' + text for text in PLAYBACK_CAVEATS) + '\n')
    print('Playback benchmark ' + scenario['status'] + ': ' + str(args.output))
    if scenario['errors']:
        print('; '.join(scenario['errors']), file=sys.stderr)
    return result


@contextmanager
def interruption_signals():
    previous = {kind: signal.getsignal(kind) for kind in (signal.SIGTERM, signal.SIGHUP)}
    def interrupt(signum, frame):
        raise KeyboardInterrupt
    try:
        for kind in previous:
            signal.signal(kind, interrupt)
        yield
    finally:
        for kind, handler in previous.items():
            signal.signal(kind, handler)


def main():
    args = parse_args()
    try:
        with interruption_signals():
            return run_playback(args)
    except FileExistsError:
        print('Output directory already exists; choose a new directory.', file=sys.stderr)
        return 2
    except OSError:
        print('Could not write playback artifacts; check output directory permissions.', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
