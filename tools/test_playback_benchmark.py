"""Playback orchestration uses invented Docker IDs and loopback HTTP fixtures."""
import io
import http.server
import threading
import json
import pathlib
import os
import signal
import subprocess
import sys
import time
import tempfile
import unittest
from unittest import mock

import playback_benchmark as play


class IsolationTests(unittest.TestCase):
    def test_explicit_cached_images_and_meaningful_phase_window_are_required(self):
        with mock.patch('sys.stderr'):
            for args in [[], ['--jellyfin-image', 'example:1', '--finstats-image', 'FinStats=example:2',
                              '--duration', '10', '--interval', '10']]:
                with self.assertRaises(SystemExit):
                    play.parse_args(['--output', '/tmp/example'] + args)
        args = play.parse_args(['--output', '/tmp/example', '--jellyfin-image', 'jellyfin/jellyfin:12.4',
                                '--finstats-image', 'FinStats=finstats:latest'])
        self.assertEqual(args.streams, 1)
        self.assertEqual(args.duration, 300)

    def test_sandbox_uses_internal_network_scratch_loopback_limits_and_no_pull(self):
        with tempfile.TemporaryDirectory() as temp:
            calls = []
            def docker(argv):
                calls.append(argv)
                if argv[:2] == ['network', 'create']:
                    return 'network-id'
                if argv[0] == 'create':
                    return 'container-id'
                if argv[0] == 'inspect' or argv[:2] == ['network', 'inspect']:
                    return 'test-run'
                return ''
            sandbox = play.Sandbox(pathlib.Path(temp), 'test-run', docker)
            sandbox.network()
            sandbox.container('jellyfin', 'example:1', {pathlib.Path(temp) / 'media': ('/media', True)}, {}, 8096)
            create = next(c for c in calls if c[0] == 'create')
            self.assertIn('--internal', calls[0])
            self.assertIn('--pull=never', create)
            self.assertNotIn('--publish', create)
            self.assertIn('--memory', create)
            self.assertIn('--cpus', create)
            self.assertIn('readonly', ' '.join(create))
            self.assertFalse(any('finstats' == arg for arg in create))
            sandbox.cleanup()
            self.assertIn(['rm', '-f', 'container-id'], calls)
            self.assertIn(['network', 'rm', 'network-id'], calls)

    def test_internal_network_api_is_forwarded_through_owned_loopback_proxy(self):
        with tempfile.TemporaryDirectory() as temp:
            def docker(argv):
                if argv[0] == 'create': return 'owned-id'
                if argv[0] == 'inspect' and '.IPAddress' in argv[2]: return '172.18.0.2'
                if argv[0] == 'inspect': return 'test-run'
                return ''
            sandbox = play.Sandbox(pathlib.Path(temp), 'test-run', docker)
            sandbox.container('jellyfin', 'example:1', {}, {}, 8096)
            with mock.patch.object(play, 'LoopbackProxy') as proxy:
                proxy.return_value.url = 'http://127.0.0.1:9999'
                self.assertEqual(sandbox.proxy_url('owned-id', 8096, 5), 'http://127.0.0.1:9999')
            sandbox.cleanup()
            proxy.return_value.close.assert_called_once_with()
            with self.assertRaises(play.PlaybackError): sandbox.proxy_url('not-ours', 8096, 5)

    def test_container_secrets_use_private_temporary_env_file(self):
        with tempfile.TemporaryDirectory() as temp:
            seen = []
            def docker(argv):
                if argv[0] == 'create':
                    self.assertNotIn('fixture-secret', ' '.join(argv))
                    file = pathlib.Path(argv[argv.index('--env-file') + 1])
                    self.assertEqual(file.stat().st_mode & 0o777, 0o600)
                    self.assertIn('JELLYFIN_API_KEY=fixture-secret', file.read_text())
                    seen.append(file)
                    return 'owned-id'
                return ''
            sandbox = play.Sandbox(pathlib.Path(temp), 'test-run', docker)
            sandbox.container('tracker', 'example:1', {}, {'JELLYFIN_API_KEY':'fixture-secret'}, 8080)
            self.assertTrue(seen)
            self.assertFalse(seen[0].exists())

    def test_cleanup_refuses_resources_whose_run_label_does_not_match(self):
        calls = []
        def docker(argv):
            calls.append(argv)
            if argv[0] == 'create': return 'container-id'
            if argv[0] == 'inspect': return 'some-other-run'
            return ''
        with tempfile.TemporaryDirectory() as temp:
            sandbox = play.Sandbox(pathlib.Path(temp), 'test-run', docker)
            sandbox.container('tracker', 'example:1', {}, {}, 8080)
            sandbox.cleanup()
        self.assertFalse(any(c[0] == 'rm' for c in calls))
        self.assertTrue(sandbox.cleanup_errors)


class JellyfinFixture(http.server.BaseHTTPRequestHandler):
    calls = []
    completed = False
    needs_refresh = False
    def log_message(self, *args): pass
    def do_GET(self): self.answer()
    def do_POST(self): self.answer()
    def answer(self):
        body = self.rfile.read(int(self.headers.get('Content-Length', 0)))
        self.calls.append((self.command, self.path, json.loads(body) if body else None))
        if self.path == '/System/Info/Public': value = {'Id': 'server-test', 'Version': 'fixture', 'StartupWizardCompleted': type(self).completed}
        elif self.path == '/Startup/Complete': type(self).completed = True; value = None
        elif self.path == '/Startup/User': value = {'Name': 'initial'}
        elif self.path == '/Users/AuthenticateByName': value = {'AccessToken': 'fixture-secret', 'User': {'Id': 'user-test'}}
        elif self.path == '/ScheduledTasks': value = [{'Key':'RefreshLibrary','State':'Idle'}]
        elif self.path == '/Library/Refresh': type(self).needs_refresh = False; value = None
        elif self.path.startswith('/Items?'): value = {'Items': [] if type(self).needs_refresh else [{'Id': 'movie-test', 'Name': 'Benchmark', 'RunTimeTicks': 9000000000}]}
        elif '/PlaybackInfo' in self.path: value = {'PlaySessionId': 'play-test', 'MediaSources': [{'Id': 'source-test', 'Container': 'mp4', 'SupportsDirectPlay': True}]}
        else: value = None
        payload = json.dumps(value).encode() if value is not None else b''
        self.send_response(200 if payload else 204); self.send_header('Content-Type', 'application/json'); self.send_header('Content-Length', str(len(payload))); self.end_headers(); self.wfile.write(payload)


class HttpLifecycleTests(unittest.TestCase):
    def setUp(self):
        JellyfinFixture.calls = []
        JellyfinFixture.completed = False
        JellyfinFixture.needs_refresh = False
        self.server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), JellyfinFixture)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True); self.thread.start()
        self.api = play.Jellyfin('http://127.0.0.1:' + str(self.server.server_port), 'device-test', timeout=1)
    def tearDown(self):
        self.server.shutdown(); self.server.server_close(); self.thread.join()

    def test_bootstrap_and_real_client_reports_target_only_disposable_server(self):
        item, server = play.bootstrap(self.api, 'invented-password', 2)
        self.assertEqual(item['Id'], 'movie-test')
        self.assertEqual(server['Id'], 'server-test')
        source, session = self.api.playback_info(item['Id'])
        self.api.report('start', item['Id'], source, session, 0, False)
        self.api.report('progress', item['Id'], source, session, 20000000, True)
        self.api.report('progress', item['Id'], source, session, 20000000, False)
        self.api.report('stop', item['Id'], source, session, 40000000, False)
        reports = [call for call in JellyfinFixture.calls if call[1].startswith('/Sessions/Playing')]
        self.assertEqual([r[1] for r in reports], ['/Sessions/Playing', '/Sessions/Playing/Progress', '/Sessions/Playing/Progress', '/Sessions/Playing/Stopped'])
        self.assertTrue(reports[1][2]['IsPaused'])
        self.assertFalse(reports[2][2]['IsPaused'])
        self.assertEqual(reports[0][2]['PlayMethod'], 'DirectPlay')
        with self.assertRaises(play.PlaybackError): self.api.request('GET', 'https://example.com/')
        with self.assertRaises(play.PlaybackError): play.Jellyfin('http://example.com:8096', 'device', 1)

    def test_loopback_proxy_forwards_actual_http_to_internal_server(self):
        proxy = play.LoopbackProxy('127.0.0.1', self.server.server_port, 1)
        try:
            api = play.Jellyfin(proxy.url, 'device-test', 1)
            self.assertEqual(api.request('GET', '/System/Info/Public')['Id'], 'server-test')
            api.authenticate('invented-password')
            self.assertEqual(api.token, 'fixture-secret')
        finally: proxy.close()

    def test_tokens_use_current_authorization_header_not_legacy_header(self):
        self.api.token = 'fixture-secret'
        headers = self.api.headers()
        self.assertIn('Token="fixture-secret"', headers['Authorization'])
        self.assertNotIn('X-Emby-Token', headers)

    def test_bootstrap_waits_for_user_api_after_early_public_status(self):
        original = self.api.request
        attempts = []
        def request(method, path, body=None):
            if method == 'GET' and path == '/Startup/User':
                attempts.append(path)
                if len(attempts) == 1: raise play.PlaybackError('disposable Jellyfin returned HTTP 503')
            return original(method, path, body)
        with mock.patch.object(self.api, 'request', side_effect=request), mock.patch.object(play.time, 'sleep'):
            movie, _ = play.bootstrap(self.api, 'invented-password', 2)
        self.assertEqual(movie['Id'], 'movie-test')
        self.assertEqual(len(attempts), 2)

    def test_bootstrap_waits_for_complete_identity_and_rereads_after_setup(self):
        original = self.api.request
        public_calls = []
        def request(method, path, body=None):
            if path == '/System/Info/Public':
                public_calls.append(path)
                if len(public_calls) == 1: return {'State':'Starting'}
            return original(method, path, body)
        with mock.patch.object(self.api, 'request', side_effect=request), mock.patch.object(play.time, 'sleep'):
            _, server = play.bootstrap(self.api, 'invented-password', 2)
        self.assertGreaterEqual(len(public_calls), 3)
        self.assertTrue(server['StartupWizardCompleted'])
        self.assertEqual(server['Id'], 'server-test')
        self.assertEqual(server['Version'], 'fixture')

    def test_empty_success_responses_raise_clean_playback_errors(self):
        with mock.patch.object(self.api, 'request', return_value=None):
            with self.assertRaises(play.PlaybackError): self.api.playback_info('movie')
        original = self.api.request
        def request(method, path, body=None):
            if path.startswith('/Items?'): return None
            return original(method, path, body)
        with mock.patch.object(self.api, 'request', side_effect=request), mock.patch.object(play.time, 'sleep'):
            with self.assertRaises(play.PlaybackError): play.bootstrap(self.api, 'invented-password', 0.01)

    def test_bootstrap_refreshes_the_new_library_after_an_initial_scan_is_idle(self):
        JellyfinFixture.needs_refresh = True
        with mock.patch.object(play.time, 'sleep'):
            movie, _ = play.bootstrap(self.api, 'invented-password', 0.02)
        self.assertEqual(movie['Id'], 'movie-test')
        paths = [call[1] for call in JellyfinFixture.calls]
        self.assertLess(paths.index('/ScheduledTasks'), paths.index('/Library/Refresh'))

    def test_transcoding_is_rejected_instead_of_mislabeled(self):
        self.api.user_id = 'user-test'
        with mock.patch.object(self.api, 'request', return_value={'PlaySessionId':'play', 'MediaSources':[{'Id':'source', 'SupportsDirectPlay':False}]}):
            with self.assertRaisesRegex(play.PlaybackError, 'DirectPlay'):
                self.api.playback_info('movie')


class WorkloadTests(unittest.TestCase):
    def test_play_pause_resume_stop_has_explicit_measurement_phases(self):
        self.assertEqual([play.phase(t, 12, 2) for t in (0, 4, 6, 8, 12)],
                         ['playing', 'paused', 'playing', 'stopped', 'stopped'])

    def test_workload_pauses_decoder_and_reports_observed_positions(self):
        api = mock.Mock()
        client = mock.Mock()
        client.snapshot.return_value = {'position_ticks': 20000000, 'frames': 20, 'decoded_seconds': 2}
        output = io.StringIO()
        workload = play.Workload([(api, client, 'movie', 'source', 'session')], 12, 2, 1, output)
        for instant in (0, 4, 6, 8): workload.step(instant)
        client.pause.assert_called_once_with()
        client.resume.assert_called_once_with()
        client.close.assert_called_once_with()
        kinds = [json.loads(line)['type'] for line in output.getvalue().splitlines()]
        self.assertIn('pause', kinds); self.assertIn('resume', kinds); self.assertIn('stop', kinds)
        self.assertIn(mock.call.report('progress', 'movie', 'source', 'session', 20000000, True), api.mock_calls)
        self.assertIn(mock.call.report('stop', 'movie', 'source', 'session', 20000000, False), api.mock_calls)

    def test_finish_is_idempotent_after_interruption(self):
        api, client = mock.Mock(), mock.Mock()
        client.snapshot.return_value = {'position_ticks': 1}
        workload = play.Workload([(api, client, 'movie', 'source', 'session')], 12, 2, 1, io.StringIO())
        workload.finish(); workload.finish()
        api.report.assert_called_once_with('stop', 'movie', 'source', 'session', 1, False)

    def test_history_checks_close_their_read_only_database_handle(self):
        with tempfile.TemporaryDirectory() as temp:
            data = pathlib.Path(temp)
            (data / 'finstats.db').touch()
            connection = mock.MagicMock()
            connection.__enter__.return_value = connection
            connection.execute.return_value.fetchone.return_value = (1, 1)
            with mock.patch.object(play.sqlite3, 'connect', return_value=connection):
                self.assertEqual(play.history(data), (1, 1))
            connection.close.assert_called_once_with()

    def test_events_share_monotonic_time_with_resource_counters(self):
        output = io.StringIO()
        workload = play.Workload([], 12, 2, 1, output)
        workload.emit('start', 0, 0, {'position_ticks': 1})
        self.assertGreater(json.loads(output.getvalue())['monotonic_s'], 0)

    def test_decoder_uses_actual_http_media_and_realtime_decoding(self):
        api = mock.Mock(token='fixture-secret')
        api.headers.return_value = {'X-Emby-Token': 'fixture-secret'}
        api.stream_url.return_value = 'http://127.0.0.1:8096/Videos/movie/stream.mp4?Static=true'
        process = mock.Mock(stdout=io.StringIO('frame=24\nout_time_us=2000000\nprogress=continue\n'))
        process.poll.return_value = None
        with mock.patch.object(play.subprocess, 'Popen', return_value=process) as popen, mock.patch.object(play, 'wait_until'):
            client = play.MediaClient('ffmpeg', api, 'movie', 'source', 'session', 60, 5)
            client.start()
            client.reader.join(timeout=1)
        command = popen.call_args.args[0]
        self.assertIn('-re', command)
        self.assertIn(api.stream_url.return_value, command)
        self.assertEqual(client.snapshot()['position_ticks'], 20000000)
        self.assertNotIn('fixture-secret', json.dumps(client.snapshot()))


class ControllerTests(unittest.TestCase):
    def test_missing_cached_image_never_creates_containers_and_keeps_failure_artifact(self):
        with tempfile.TemporaryDirectory() as temp:
            args = play.parse_args(['--output', temp + '/out', '--jellyfin-image', 'example:missing', '--finstats-image', 'FinStats=example:2'])
            with mock.patch.object(play.idle, 'require_local_docker'), mock.patch.object(play, 'docker_command', side_effect=play.PlaybackError('image unavailable')), mock.patch.object(play, 'Sandbox') as sandbox, mock.patch('sys.stdout', new=io.StringIO()), mock.patch('sys.stderr', new=io.StringIO()):
                self.assertEqual(play.run_playback(args), 1)
            sandbox.assert_not_called()
            artifact = json.loads((args.output / 'scenario.json').read_text())
            self.assertEqual(artifact['status'], 'failed')
            self.assertNotIn('password', artifact)
            self.assertNotIn('token', artifact)


    def test_scratch_removal_failure_is_recorded_without_losing_scenario(self):
        with tempfile.TemporaryDirectory() as temp:
            args = play.parse_args(['--output', temp + '/out', '--jellyfin-image', 'example:1', '--finstats-image', 'FinStats=example:2'])
            scratch = pathlib.Path(temp) / 'scratch'; scratch.mkdir()
            with mock.patch.object(play.idle, 'require_local_docker'), mock.patch.object(play, 'docker_command', return_value='image'), mock.patch.object(play.tempfile, 'mkdtemp', return_value=str(scratch)), mock.patch.object(play, 'generate_media', side_effect=play.PlaybackError('fixture encoding failed')), mock.patch.object(play.shutil, 'rmtree', side_effect=PermissionError('fixture')), mock.patch('sys.stdout', new=io.StringIO()), mock.patch('sys.stderr', new=io.StringIO()):
                self.assertEqual(play.run_playback(args), 1)
            scenario = json.loads((args.output / 'scenario.json').read_text())
            self.assertEqual(scenario['scratch_retained'], str(scratch))
            self.assertTrue(scenario['cleanup_errors'])

    def test_sampler_interruption_skips_history_validation_and_keeps_interrupted_status(self):
        with tempfile.TemporaryDirectory() as temp:
            args = play.parse_args(['--output', temp + '/out', '--jellyfin-image', 'example:1', '--finstats-image', 'FinStats=example:2', '--warmup', '0'])
            sandbox = mock.Mock(cleanup_errors=[])
            sandbox.container.side_effect = [('jf-id', 'jf-name'), ('tracker-id', 'tracker-name')]
            sandbox.proxy_url.return_value = 'http://127.0.0.1:9999'
            client = mock.Mock()
            client.snapshot.return_value = {'position_ticks': 1, 'frames': 1, 'decoded_seconds': 0.0000001}
            workload = mock.Mock(errors=[], events=[], started_monotonic_s=1)
            def media(ffmpeg, destination, duration): destination.write_bytes(b'invented media')
            with mock.patch.object(play.idle, 'require_local_docker'), mock.patch.object(play, 'docker_command', return_value='image-id'), mock.patch.object(play, 'Sandbox', return_value=sandbox), mock.patch.object(play, 'generate_media', side_effect=media), mock.patch.object(play, 'bootstrap', return_value=({'Id':'movie'}, {'Id':'server','Version':'fixture'})), mock.patch.object(play, 'wait_tracker'), mock.patch.object(play.Jellyfin, 'authenticate'), mock.patch.object(play.Jellyfin, 'playback_info', return_value=('source','session')), mock.patch.object(play.Jellyfin, 'report'), mock.patch.object(play, 'MediaClient', return_value=client), mock.patch.object(play, 'Workload', return_value=workload), mock.patch.object(play.idle, 'run', return_value=130), mock.patch.object(play, 'history', side_effect=AssertionError('post-interruption validation ran')), mock.patch('sys.stdout', new=io.StringIO()):
                self.assertEqual(play.run_playback(args), 130)
            self.assertEqual(json.loads((args.output / 'scenario.json').read_text())['status'], 'interrupted')
            sandbox.cleanup.assert_called_once_with()


class SignalTests(unittest.TestCase):
    def test_term_and_hup_trigger_cleanup_and_restore_previous_handlers(self):
        for kind in (signal.SIGTERM, signal.SIGHUP):
            source = "import os,signal,sys; import playback_benchmark as p\nold=signal.getsignal(signal.SIGTERM)\ntry:\n with p.interruption_signals():\n  print('ready',flush=True)\n  signal.pause()\nexcept KeyboardInterrupt:\n print('cleaned',flush=True)\nassert signal.getsignal(signal.SIGTERM)==old\n"
            process = subprocess.Popen([sys.executable, '-u', '-c', source], cwd=pathlib.Path(play.__file__).parent,
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                self.assertEqual(process.stdout.readline().strip(), 'ready')
                os.kill(process.pid, kind)
                stdout, stderr = process.communicate(timeout=3)
                self.assertEqual(process.returncode, 0, stderr)
                self.assertIn('cleaned', stdout)
            finally:
                if process.poll() is None: process.kill(); process.wait()


if __name__ == '__main__':
    unittest.main()
