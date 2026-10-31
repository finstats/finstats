#!/usr/bin/env python3
"""Draw the README's measured idle comparison from sanitized public measurements.

Input: .github/idle-benchmark.json, never the private raw sampler output.
Output: .github/idle-benchmark.svg and the README block between benchmark markers.
Run normally to regenerate, or --check to detect stale artifacts without writing.
"""
import argparse
from datetime import datetime, timezone
from html import escape
import json
import math
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent.parent
START = '<!-- benchmark: tools/make-benchmark-chart.py writes this from .github/idle-benchmark.json -->'
END = '<!-- /benchmark -->'
PICTURE = '.github/idle-benchmark.svg'
PLAYBACK_START = '<!-- playback-benchmark: tools/make-benchmark-chart.py writes this from .github/playback-benchmark.json -->'
PLAYBACK_END = '<!-- /playback-benchmark -->'
COMBINED_START = '<!-- resource-benchmark: tools/make-benchmark-chart.py writes this from sanitized benchmark JSON -->'
COMBINED_END = '<!-- /resource-benchmark -->'


def validate(data):
    allowed = {'format_version', 'started_utc', 'finished_utc', 'duration_s', 'interval_s', 'finstats_revision', 'stacks'}
    fields = {'name', 'version', 'version_provenance', 'containers', 'cpu_mean_pct_one_core', 'working_set_mean_bytes', 'cpu_p90_pct_one_core', 'cpu_peak_pct_one_core', 'working_set_p90_bytes', 'working_set_peak_bytes', 'valid_intervals', 'memory_endpoints'}
    def number(value, minimum=0):
        return type(value) in (int, float) and math.isfinite(value) and value >= minimum
    def text(value, maximum):
        return isinstance(value, str) and 0 < len(value) <= maximum and not any(ord(c) < 32 or ord(c) == 127 for c in value)
    if not isinstance(data, dict) or set(data) - (allowed | {'scenario', 'playback'}) or not allowed <= set(data) or type(data['format_version']) is not int or data['format_version'] != 2:
        raise ValueError('use only the documented sanitized measurement fields')
    if not number(data['duration_s'], 0.001) or not number(data['interval_s'], 0.001):
        raise ValueError('duration and interval must be finite and positive')
    ratio = data['duration_s'] / data['interval_s']
    if not math.isfinite(ratio) or not math.isclose(ratio, round(ratio), abs_tol=1e-9) or not 1 <= ratio <= 100000:
        raise ValueError('duration must contain a whole number of intervals')
    try:
        start, end = [datetime.fromisoformat(data[key].replace('Z', '+00:00')) for key in ('started_utc', 'finished_utc')]
        tolerance = 1.000001 if start.microsecond == end.microsecond == 0 else 0.000001
        if start.utcoffset() != timezone.utc.utcoffset(start) or end.utcoffset() != timezone.utc.utcoffset(end) or not math.isclose((end - start).total_seconds(), data['duration_s'], rel_tol=0, abs_tol=tolerance):
            raise ValueError()
    except (ValueError, TypeError, AttributeError):
        raise ValueError('UTC timestamps must describe the measured duration') from None
    if not isinstance(data['finstats_revision'], str) or not re.fullmatch(r'[a-f0-9]{40}', data['finstats_revision']):
        raise ValueError('a full tested FinStats revision is required')
    scenario = data.get('scenario', 'idle')
    if scenario not in ('idle', 'playback'):
        raise ValueError('scenario must be idle or playback')
    playback = data.get('playback')
    if scenario == 'playback':
        if not isinstance(playback, dict) or set(playback) != {'streams', 'method', 'verification'} or type(playback['streams']) is not int or not 1 <= playback['streams'] <= 100:
            raise ValueError('playback requires a public stream count, method and verification')
        if playback['method'] not in ('unverified', 'DirectPlay', 'DirectStream', 'Transcode', 'mixed') or playback['verification'] != 'operator-confirmed':
            raise ValueError('playback method/verification must use explicit public labels')
    elif playback is not None:
        raise ValueError('idle input cannot contain a playback workload')
    stacks = data['stacks']
    if not isinstance(stacks, list) or not 1 <= len(stacks) <= 6:
        raise ValueError('supply one to six measured stacks')
    names = set()
    for stack in stacks:
        if not isinstance(stack, dict) or set(stack) != fields or not text(stack['name'], 32) or stack['name'] in names:
            raise ValueError('stacks require unique short names and only public fields')
        names.add(stack['name'])
        if stack['version'] is not None and not text(stack['version'], 24):
            raise ValueError('versions must be verified labels or null')
        if stack['version_provenance'] not in ('runtime_cli','application_package','database_cli','unavailable') or ((stack['version'] is None) != (stack['version_provenance']=='unavailable')):
            raise ValueError('published versions require explicit observed provenance')
        if type(stack['containers']) is not int or not 1 <= stack['containers'] <= 100:
            raise ValueError('each stack must count all measured containers')
        if any(not number(stack[key]) for key in ('cpu_mean_pct_one_core', 'cpu_p90_pct_one_core', 'cpu_peak_pct_one_core', 'working_set_mean_bytes', 'working_set_p90_bytes', 'working_set_peak_bytes')):
            raise ValueError('chart means must be finite and nonnegative')
        if stack['cpu_peak_pct_one_core'] < max(stack['cpu_mean_pct_one_core'],stack['cpu_p90_pct_one_core']) or stack['working_set_peak_bytes'] < max(stack['working_set_mean_bytes'],stack['working_set_p90_bytes']):
            raise ValueError('means and P90 cannot exceed their measured peaks')
        if type(stack['valid_intervals']) is not int or stack['valid_intervals'] != round(ratio) or type(stack['memory_endpoints']) is not int or stack['memory_endpoints'] != round(ratio) + 1:
            raise ValueError('publish only complete interval and endpoint coverage')
    return data


def label(stack):
    return stack['name'] + (' ' + stack['version'] if stack['version'] else '')


def stated(stack):
    return f"{label(stack)}: CPU mean {stack['cpu_mean_pct_one_core']:.3f}%, P90 {stack['cpu_p90_pct_one_core']:.3f}%, peak {stack['cpu_peak_pct_one_core']:.3f}%; working set mean {stack['working_set_mean_bytes']/1048576:.2f} MiB, P90 {stack['working_set_p90_bytes']/1048576:.2f} MiB, peak {stack['working_set_peak_bytes']/1048576:.2f} MiB"


def limit(value):
    if not value:
        return 1
    step = 10 ** math.floor(math.log10(value))
    return math.ceil(value / step) * step


def workload_text(data):
    playback = data['playback']
    count = playback['streams']
    method = 'play method unverified' if playback['method'] == 'unverified' else 'reported ' + playback['method']
    return f'{count} operator-confirmed stream{"s" if count != 1 else ""} · {method}'


def picture(data, limits=None):
    validate(data)
    stacks = data['stacks']
    scenario = data.get('scenario', 'idle')
    header = 122 if scenario == 'playback' else 98
    width, track, row = 680, 632, 78
    panel_height = 72 + row * len(stacks)
    height = header + panel_height * 2 + 108
    font = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"
    date = datetime.fromisoformat(data['started_utc'].replace('Z', '+00:00')).strftime('%d %B %Y').lstrip('0')
    description = '; '.join(stated(stack) for stack in stacks) + '. Full stack totals including workers and databases. Observational deployment comparison.'
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-labelledby="title description">',
           f'<title id="title">Measured {scenario} CPU and memory on the same host</title>', f'<desc id="description">{escape(description)}</desc>',
           '<style>',
           'svg { color-scheme:light; --surface:#ffffff; --border:#d0d7de; --foreground:#1f2328; --muted:#59636e; --track:#f0edf8; --series-1:#8a5cf5; --series-2:#3b73d9; --series-3:#188d7b; }',
           '@media (prefers-color-scheme: dark) { svg { color-scheme:dark; --surface:#0d1117; --border:#3d444d; --foreground:#f0f6fc; --muted:#9198a1; --track:#242035; --series-1:#a78bfa; --series-2:#74a7ff; --series-3:#3ac3a6; } }',
           '.card { fill:var(--surface); stroke:var(--border); }',
           f'text {{ font-family:{font}; fill:var(--foreground); }}',
           '.title { font-size:22px; font-weight:650; } .heading { font-size:17px; font-weight:650; }',
           '.name,.value { font-size:15px; font-weight:600; } .value { font-variant-numeric:tabular-nums; }',
           '.sub,.axis,.count { fill:var(--muted); font-size:12px; } .sub { font-size:13px; }',
           '.range { opacity:.22; } .p90 { stroke:var(--foreground); stroke-width:2; } .track { fill:var(--track); } .series-1 { fill:var(--series-1); } .series-2 { fill:var(--series-2); } .series-3 { fill:var(--series-3); }',
           '</style>',
           f'<rect class="card" x="0.5" y="0.5" width="{width-1}" height="{height-1}" rx="12"/>',
           f'<text class="title" x="24" y="38">{scenario.capitalize()}, on the same host</text>',
           f'<text class="sub" x="24" y="62">{escape(date)} · {data["duration_s"]:g} seconds · Simultaneous deployed stacks</text>']
    if scenario == 'playback':
        out.append(f'<text class="sub" x="24" y="84">{escape(workload_text(data))}</text>')
    for panel, (heading, key, p90_key, peak_key, divisor, unit) in enumerate([
            ('CPU usage', 'cpu_mean_pct_one_core', 'cpu_p90_pct_one_core', 'cpu_peak_pct_one_core', 1, '% of one core'),
            ('Working set', 'working_set_mean_bytes', 'working_set_p90_bytes', 'working_set_peak_bytes', 1048576, 'MiB')]):
        top = header + panel * panel_height
        maximum = (limits or {}).get(key,limit(max(stack[peak_key] / divisor for stack in stacks)))
        out += [f'<text class="heading" x="24" y="{top}">{heading}</text>', f'<text class="sub" x="656" y="{top}" text-anchor="end">{unit}</text>']
        for index, stack in enumerate(stacks):
            y = top + 29 + index * row
            value = stack[key] / divisor
            formatted = f'{value:.3f}%' if panel == 0 else f'{value:.2f} MiB'
            p90, peak = stack[p90_key]/divisor, stack[peak_key]/divisor
            fmt = lambda v: f'{v:.3f}%' if panel == 0 else f'{v:.2f} MiB'
            spike = f' ({data["interval_s"]:g}s interval)' if panel == 0 else ' (endpoint)'
            count = stack['containers']
            out += [f'<text class="name" x="24" y="{y}">{escape(label(stack))}</text>',
                    f'<text class="count" x="210" y="{y}">{count} container{"s" if count != 1 else ""}</text>',
                    f'<text class="value" x="656" y="{y}" text-anchor="end">Mean {formatted}</text>',
                    f'<rect class="track" x="24" y="{y+10}" width="{track}" height="16" rx="8"/>',
                    f'<rect class="range series-{index % 3 + 1}" x="24" y="{y+10}" width="{track*(peak/maximum):.5f}" height="16" rx="8"/>',
                    f'<rect class="bar series-{index % 3 + 1}" x="24" y="{y+10}" width="{track*(value/maximum):.5f}" height="16" rx="8"/>',
                    f'<line class="p90" x1="{24+track*(p90/maximum):.5f}" x2="{24+track*(p90/maximum):.5f}" y1="{y+6}" y2="{y+30}"/>',
                    f'<circle class="peak series-{index % 3 + 1}" cx="{24+track*(peak/maximum):.5f}" cy="{y+18}" r="4"/>',
                    f'<text class="sub" x="24" y="{y+49}">P90 {fmt(p90)} · Peak {fmt(peak)}{spike}</text>']
        axis_y = top + 35 + row * len(stacks)
        out += [f'<text class="axis" x="24" y="{axis_y}">0</text>',
                f'<text class="axis" x="656" y="{axis_y}" text-anchor="end">{maximum:g}{"%" if panel == 0 else " MiB"}</text>']
    footer = height - 77
    intervals = round(data['duration_s'] / data['interval_s'])
    out += [f'<text class="sub" x="24" y="{footer}">Complete coverage: {intervals}/{intervals} CPU intervals · {intervals+1} memory endpoints</text>',
            f'<text class="axis" x="24" y="{footer+20}">Full stacks include workers and databases. Observational comparison.</text>',
            f'<text class="axis" x="24" y="{footer+40}">Versions: observed runtime CLI and application packages; tags may differ.</text>',
            f'<text class="axis" x="24" y="{footer+60}">FinStats tested revision: {data["finstats_revision"]}</text>', '</svg>']
    return '\n'.join(out) + '\n'


def section(data):
    scenario = data.get('scenario', 'idle')
    alt = 'Measured ' + scenario + ' comparison, ' + data['started_utc'][:10] + ': ' + '; '.join(stated(stack) for stack in data['stacks']) + '. Observational full-stack totals.'
    if scenario == 'playback':
        alt += ' ' + workload_text(data) + '.'
    start, end = (PLAYBACK_START, PLAYBACK_END) if scenario == 'playback' else (START, END)
    picture_path = '.github/playback-benchmark.svg' if scenario == 'playback' else PICTURE
    first = datetime.fromisoformat(data['started_utc'].replace('Z', '+00:00'))
    last = datetime.fromisoformat(data['finished_utc'].replace('Z', '+00:00'))
    finish = last.strftime('%H:%M:%S') if first.date() == last.date() else last.strftime('%d %B %Y, %H:%M:%S')
    window = first.strftime('%d %B %Y, %H:%M:%S') + ' to ' + finish + ' UTC'
    finstats = next((label(stack) for stack in data['stacks'] if stack['name'] == 'FinStats'), 'FinStats')
    caption = '<p>Measured UTC endpoints: <strong>' + escape(window) + '</strong>. ' + escape(finstats) + '; tested revision <code>' + data['finstats_revision'] + '</code>.</p>'
    return start + '\n' + caption + '\n<p align="center"><img src="' + picture_path + '" width="680" alt="' + escape(alt, quote=True) + '"></p>\n' + end


def validate_pair(idle, playback):
    validate(idle);validate(playback)
    if idle.get('scenario','idle') != 'idle' or playback.get('scenario') != 'playback':
        raise ValueError('both scenarios are required')
    if idle['finstats_revision'] != playback['finstats_revision']:
        raise ValueError('scenarios must identify the same tested FinStats revision')
    if [(s['name'],s['version'],s['containers']) for s in idle['stacks']] != [(s['name'],s['version'],s['containers']) for s in playback['stacks']]:
        raise ValueError('paired scenarios must identify the same application stacks')


def composite_picture(idle, playback):
    """Selected layout A: one app-focused panel, paired states and CPU/RAM columns."""
    validate_pair(idle,playback)
    width=728;row_height=140;height=184+row_height*len(idle['stacks'])
    columns=[('cpu',180,256.5,'cpu_mean_pct_one_core','cpu_p90_pct_one_core','cpu_peak_pct_one_core',1),
             ('ram',454.5,256.5,'working_set_mean_bytes','working_set_p90_bytes','working_set_peak_bytes',1048576)]
    scales={metric:limit(max(stack[peak_key]/divisor for data in (idle,playback) for stack in data['stacks']))
            for metric,x,w,mean_key,p90_key,peak_key,divisor in columns}
    description='Idle: '+'; '.join(stated(stack) for stack in idle['stacks'])+'. Playback: '+workload_text(playback)+'. '+'; '.join(stated(stack) for stack in playback['stacks'])+'. Both are observational full-stack measurements.'
    font="-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"
    out=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-labelledby="benchmark-title benchmark-description">',
         '<title id="benchmark-title">Paired idle and playback resources per application</title>',
         '<desc id="benchmark-description">'+escape(description)+'</desc>',
         '<style>',
         'svg { --surface:#ffffff; --border:#d0d7de; --foreground:#1f2328; --muted:#59636e; --track:#f0edf8; --series-1:#8a5cf5; --series-2:#3b73d9; --series-3:#188d7b; }',
         '@media (prefers-color-scheme: dark) { svg { --surface:#0d1117; --border:#3d444d; --foreground:#f0f6fc; --muted:#9198a1; --track:#242035; --series-1:#a78bfa; --series-2:#74a7ff; --series-3:#3ac3a6; } }',
         f'text {{ font-family:{font}; fill:var(--foreground); }}',
         '.card { fill:var(--surface); stroke:var(--border); } .title { font-size:20px; font-weight:650; }',
         '.name,.value { font-size:14px; font-weight:650; } .value { font-variant-numeric:tabular-nums; }',
         '.sub,.tag,.tail,.head { font-size:12px; fill:var(--muted); } .tag { font-weight:400; } .head { font-weight:600; }',
         '.rule { stroke:var(--border); } .track { fill:var(--track); } .range { opacity:.2; } .idle { opacity:.45; }',
         '.p90 { stroke:var(--foreground); stroke-width:2; } .series-1 { fill:var(--series-1); } .series-2 { fill:var(--series-2); } .series-3 { fill:var(--series-3); }',
         '</style>',f'<rect class="card" x="0.5" y="0.5" width="{width-1}" height="{height-1}" rx="14"/>',
         '<text class="title" x="17" y="36">Paired per app</text>',
         f'<text class="sub" x="17" y="56">Idle and {playback["playback"]["streams"]} operator-confirmed stream{"s" if playback["playback"]["streams"]!=1 else ""}, on the same host</text>',
         '<text class="sub" x="17" y="80">Solid: mean · Tick: P90 · Dot: peak</text>',
         f'<text class="sub" x="711" y="80" text-anchor="end">Shared scales: CPU 0 to {scales["cpu"]:g}%; RAM 0 to {scales["ram"]:g} MiB</text>',
         '<text class="head" x="180" y="104">CPU · % of one core</text>',
         '<text class="head" x="454.5" y="104">Working set · MiB</text>']
    for index,app in enumerate(idle['stacks']):
        top=116+index*row_height;series=index%3+1
        out += [f'<line class="rule" x1="17" x2="711" y1="{top}" y2="{top}"/>',
                f'<circle class="series-{series}" cx="21" cy="{top+19}" r="4"/>',
                f'<text class="name" x="30" y="{top+23}">{escape(app["name"])}<tspan class="tag"> {escape(app["version"] or "")}</tspan></text>',
                f'<text class="sub" x="17" y="{top+43}">{app["containers"]} container{"s" if app["containers"]!=1 else ""}</text>']
        for metric,x,bar_width,mean_key,p90_key,peak_key,divisor in columns:
            for state_index,(state,data) in enumerate([('idle',idle),('playback',playback)]):
                stack=data['stacks'][index];mean,p90,peak=[stack[key]/divisor for key in (mean_key,p90_key,peak_key)]
                maximum=scales[metric];y=top+23+state_index*60
                formatted=lambda value: f'{value:.3f}%' if metric=='cpu' else f'{value:.2f} MiB'
                attrs=f'data-app="{escape(app["name"],quote=True)}" data-scenario="{state}" data-metric="{metric}"'
                out += [f'<text class="sub" x="{x}" y="{y}">{state.capitalize()}</text>',
                        f'<text class="value" x="{x+bar_width}" y="{y}" text-anchor="end">{formatted(mean)}</text>',
                        f'<rect class="track" x="{x}" y="{y+9}" width="{bar_width}" height="9" rx="4.5"/>',
                        f'<rect class="range series-{series}" x="{x}" y="{y+9}" width="{bar_width*peak/maximum:.5f}" height="9" rx="4.5"/>',
                        f'<rect class="bar series-{series} {"idle" if state=="idle" else "playing"}" {attrs} x="{x}" y="{y+9}" width="{bar_width*mean/maximum:.5f}" height="9" rx="4.5"/>',
                        f'<line class="p90" x1="{x+bar_width*p90/maximum:.5f}" x2="{x+bar_width*p90/maximum:.5f}" y1="{y+6}" y2="{y+21}"/>',
                        f'<circle class="peak series-{series}" cx="{x+bar_width*peak/maximum:.5f}" cy="{y+13.5}" r="3.5"/>',
                        f'<text class="tail" x="{x}" y="{y+38}">P90 {formatted(p90)} · Peak {formatted(peak)}</text>']
    footer=126+row_height*len(idle['stacks'])
    date=datetime.fromisoformat(idle['started_utc'].replace('Z','+00:00')).strftime('%d %B %Y').lstrip('0')
    duration=f'{idle["duration_s"]:g}s idle / {playback["duration_s"]:g}s playback'
    method='Playback method unverified' if playback['playback']['method']=='unverified' else 'Playback '+playback['playback']['method']+' (operator report)'
    out += [f'<text class="sub" x="17" y="{footer}">{date} · {duration} · Full stacks including databases and workers.</text>',
            f'<text class="sub" x="17" y="{footer+19}">P90: nearest rank. CPU peak: highest {idle["interval_s"]:g}s interval. RAM peak: endpoint. {escape(method)}.</text>',
            f'<text class="sub" x="17" y="{footer+38}">Observed runtime/package versions. FinStats tested revision: {idle["finstats_revision"]}</text>', '</svg>']
    return '\n'.join(out)+'\n'


def combined_section(idle,playback):
    validate_pair(idle,playback)
    alt='Idle: '+'; '.join(stated(stack) for stack in idle['stacks'])+'. Playback: '+'; '.join(stated(stack) for stack in playback['stacks'])+'. '+workload_text(playback)+'. Observational full-stack totals.'
    caption='<p>Idle UTC endpoints: '+escape(idle['started_utc'])+' to '+escape(idle['finished_utc'])+'; playback: '+escape(playback['started_utc'])+' to '+escape(playback['finished_utc'])+'. FinStats tested revision <code>'+idle['finstats_revision']+'</code>.</p>'
    return COMBINED_START+'\n'+caption+'\n<p align="center"><img src=".github/benchmark.svg" width="728" alt="'+escape(alt,quote=True)+'"></p>\n'+COMBINED_END


def main(argv=None, root=ROOT):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--scenario', choices=('idle', 'playback'), help='legacy per-scenario README block')
    args = parser.parse_args(argv)
    try:
        readme = (root / 'README.md').read_text()
        if args.scenario is None and COMBINED_START in readme:
            idle=validate(json.loads((root/'.github'/'idle-benchmark.json').read_text()))
            playback=validate(json.loads((root/'.github'/'playback-benchmark.json').read_text()))
            want,drawn=combined_section(idle,playback),composite_picture(idle,playback)
            start,end,image_path=COMBINED_START,COMBINED_END,'.github/benchmark.svg'
        else:
            scenario=args.scenario or 'idle'
            data=validate(json.loads((root/'.github'/(scenario+'-benchmark.json')).read_text()))
            if data.get('scenario','idle')!=scenario: raise ValueError('input scenario does not match selected output')
            want,drawn=section(data),picture(data)
            start,end=(PLAYBACK_START,PLAYBACK_END) if scenario=='playback' else (START,END)
            image_path='.github/'+scenario+'-benchmark.svg'
        match=re.search(re.escape(start)+'.*?'+re.escape(end),readme,re.S)
        if not match: raise ValueError('README has no benchmark marker block')
        image=root/image_path
        stale=[name for name,old,new in [('README.md',match.group(0),want),(image_path,image.read_text() if image.exists() else '',drawn)] if old!=new]
        if args.check:
            if stale:
                print('out of date: ' + ', '.join(stale), file=sys.stderr)
                return 1
        else:
            (root / 'README.md').write_text(readme[:match.start()] + want + readme[match.end():])
            image.write_text(drawn)
        return 0
    except (OSError, ValueError, KeyError, TypeError):
        print('Invalid or unreadable sanitized benchmark input or README markers.', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
