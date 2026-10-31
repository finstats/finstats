"""The public benchmark graphic accepts complete, sanitized measurements only."""
import importlib.util
import json
import re
from pathlib import Path
import tempfile
import unittest
import xml.etree.ElementTree as ET

SPEC = importlib.util.spec_from_file_location('benchmark_chart', Path(__file__).with_name('make-benchmark-chart.py'))
chart = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(chart)


def fixture():
    return {'format_version': 2, 'started_utc':'2026-10-31T00:36:33Z', 'finished_utc':'2026-10-31T00:41:33Z',
            'duration_s':300, 'interval_s':10, 'finstats_revision':'d' * 40,
            'stacks':[{'name':name, 'version':'2.3.1' if i == 0 else None, 'version_provenance':'runtime_cli' if i==0 else 'unavailable','containers':i + 1,
                       'cpu_mean_pct_one_core':cpu,'cpu_p90_pct_one_core':cpu*1.1,'cpu_peak_pct_one_core':cpu*1.2,
                       'working_set_mean_bytes':memory * 1048576,'working_set_p90_bytes':memory*1.1*1048576,'working_set_peak_bytes':memory*1.2*1048576,
                       'valid_intervals':30, 'memory_endpoints':31}
                      for i, (name, cpu, memory) in enumerate([('FinStats',2,64),('Jellystat',4,320),('Streamystats',10,640)])]}


class GraphicTests(unittest.TestCase):
    def test_bars_use_linear_zero_based_axes_and_correct_units(self):
        data = chart.validate(fixture())
        svg = chart.picture(data)
        document = ET.fromstring(svg)
        bars = [element for element in document.iter() if element.get('class','').startswith('bar series-')]
        self.assertEqual(len(bars), 6)
        cpu_widths = [float(bar.get('width')) for bar in bars[:3]]
        self.assertAlmostEqual(cpu_widths[1] / cpu_widths[0], 2, places=4)
        self.assertAlmostEqual(cpu_widths[2] / cpu_widths[0], 5, places=4)
        memory_widths = [float(bar.get('width')) for bar in bars[3:]]
        self.assertAlmostEqual(memory_widths[1] / memory_widths[0], 5, places=4)
        self.assertAlmostEqual(memory_widths[2] / memory_widths[0], 10, places=4)
        self.assertIn('800 MiB', svg)
        self.assertIn('64.00 MiB', svg)
        self.assertIn('2.000%', svg)
        self.assertIn('prefers-color-scheme: dark', svg)
        self.assertIn('<desc', svg)
        self.assertEqual(svg, chart.picture(data))

    def test_fallback_theme_uses_literal_light_and_dark_token_colors(self):
        svg = chart.picture(fixture())
        style = ET.fromstring(svg).find('{http://www.w3.org/2000/svg}style').text
        light = re.search(r'svg\s*\{([^}]+)\}', style)[1]
        dark = re.search(r'@media \(prefers-color-scheme: dark\)\s*\{\s*svg\s*\{([^}]+)\}', style)[1]
        for css, expected in [(light, {'surface':'#ffffff','foreground':'#1f2328'}),
                              (dark, {'surface':'#0d1117','foreground':'#f0f6fc'})]:
            tokens = dict(re.findall(r'--([a-z0-9-]+):\s*([^;]+)', css))
            for key, color in expected.items(): self.assertEqual(tokens[key], color)
            for key in ('surface','foreground','track','series-1','series-2','series-3'):
                self.assertRegex(tokens[key], r'^#[a-f0-9]{6}$')

    def test_generated_caption_keeps_time_version_and_revision_current(self):
        data = fixture();data['finstats_revision'] = 'a' * 40;data['stacks'][0]['version'] = '2.3.2'
        data['started_utc'] = '2026-10-31T09:49:32Z';data['finished_utc'] = '2026-10-31T09:54:32Z'
        section = chart.section(chart.validate(data))
        self.assertIn('09:49:32', section)
        self.assertIn('09:54:32', section)
        self.assertIn('a' * 40, section)
        self.assertIn('FinStats 2.3.2', section)

    def test_partial_nonfinite_or_private_input_cannot_be_published(self):
        for mutate in [lambda d:d['stacks'][0].update(valid_intervals=29),
                       lambda d:d['stacks'][0].update(memory_endpoints=30),
                       lambda d:d['stacks'][0].update(cpu_mean_pct_one_core=float('nan')),
                       lambda d:d['stacks'][0].update(working_set_mean_bytes=-1),
                       lambda d:d.update(host='private-host'),
                       lambda d:d.update(finished_utc='2026-10-31T00:41:35Z')]:
            data = fixture(); mutate(data)
            with self.subTest(data=data), self.assertRaises(ValueError): chart.validate(data)

    def test_whole_second_utc_labels_can_differ_from_monotonic_duration_by_one_second(self):
        data = fixture();data['finished_utc'] = '2026-10-31T00:41:32Z'
        self.assertEqual(chart.validate(data)['duration_s'], 300)

    def test_playback_chart_discloses_operator_confirmation_and_unverified_method(self):
        data = fixture();data['scenario'] = 'playback'
        data['playback'] = {'streams':1, 'method':'unverified', 'verification':'operator-confirmed'}
        svg = chart.picture(chart.validate(data))
        self.assertIn('Playback, on the same host', svg)
        self.assertIn('1 operator-confirmed stream', svg)
        self.assertIn('play method unverified', svg)
        self.assertNotIn('DirectPlay', svg)
        self.assertIn('.github/playback-benchmark.svg', chart.section(data))
        data['playback']['title'] = 'private-title'
        with self.assertRaises(ValueError): chart.validate(data)

    def test_stranger_text_is_xml_escaped_and_alt_text_retains_values(self):
        data = fixture();data['stacks'][1]['name'] = 'A < B & C'
        svg = chart.picture(chart.validate(data))
        ET.fromstring(svg)
        self.assertIn('A &lt; B &amp; C', svg)
        self.assertIn('2.000%', chart.section(data))
        self.assertIn('64.00 MiB', chart.section(data))

    def test_composite_uses_shared_peak_axes_and_preserves_all_statistics(self):
        idle = fixture();playing = fixture();playing['scenario']='playback';playing['playback']={'streams':1,'method':'unverified','verification':'operator-confirmed'}
        for stack in playing['stacks']:
            for key in ('cpu_mean_pct_one_core','cpu_p90_pct_one_core','cpu_peak_pct_one_core'):stack[key]*=2
        svg=chart.composite_picture(idle,playing);document=ET.fromstring(svg)
        bars=[node for node in document.iter() if node.get('class','').startswith('bar series-')]
        self.assertEqual(len(bars),12)
        paired={node.get('data-scenario'):node for node in bars if node.get('data-app')=='FinStats' and node.get('data-metric')=='cpu'}
        self.assertAlmostEqual(float(paired['playback'].get('width'))/float(paired['idle'].get('width')),2,places=4)
        self.assertIn('P90',svg);self.assertIn('Peak',svg);self.assertIn('10s interval',svg)
        self.assertEqual(len([n for n in document.iter() if n.get('class')=='p90']),12)
        self.assertEqual(len([n for n in document.iter() if n.get('class','').startswith('peak series-')]),12)

    def test_selected_app_dashboard_has_one_background_and_explicit_workloads(self):
        idle=fixture();playing=fixture();playing['scenario']='playback';playing['playback']={'streams':1,'method':'unverified','verification':'operator-confirmed'}
        svg=chart.composite_picture(idle,playing);document=ET.fromstring(svg)
        self.assertEqual(len([n for n in document.iter() if n.get('class')=='card']),1)
        description=document.find('{http://www.w3.org/2000/svg}desc').text
        self.assertIn('Idle:',description);self.assertIn('Playback:',description);self.assertIn('operator-confirmed',description)
        playing['finstats_revision']='a'*40
        with self.assertRaises(ValueError):chart.composite_picture(idle,playing)
        with self.assertRaises(ValueError):chart.combined_section(idle,playing)

    def test_combined_cli_checks_one_svg_and_its_accessible_readme_block(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);(root/'.github').mkdir();idle=fixture();playing=fixture();playing['scenario']='playback';playing['playback']={'streams':1,'method':'unverified','verification':'operator-confirmed'}
            for name,data in [('idle',idle),('playback',playing)]: (root/'.github'/(name+'-benchmark.json')).write_text(json.dumps(data))
            (root/'README.md').write_text(chart.COMBINED_START+'\nstale\n'+chart.COMBINED_END)
            self.assertEqual(chart.main([],root),0);self.assertEqual(chart.main(['--check'],root),0)
            self.assertTrue((root/'.github'/'benchmark.svg').exists())
            self.assertIn('P90',(root/'README.md').read_text())

    def test_playback_cli_writes_and_checks_its_own_graph_and_marker(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp);(root / '.github').mkdir()
            data = fixture();data['scenario']='playback';data['playback']={'streams':1,'method':'unverified','verification':'operator-confirmed'}
            (root / '.github' / 'playback-benchmark.json').write_text(json.dumps(data))
            (root / 'README.md').write_text(chart.PLAYBACK_START + '\nstale\n' + chart.PLAYBACK_END)
            self.assertEqual(chart.main(['--scenario','playback'], root), 0)
            self.assertEqual(chart.main(['--scenario','playback','--check'], root), 0)
            self.assertTrue((root / '.github' / 'playback-benchmark.svg').exists())
            self.assertFalse((root / '.github' / 'idle-benchmark.svg').exists())
            self.assertIn('Playback, on the same host', (root / '.github' / 'playback-benchmark.svg').read_text())

    def test_check_detects_stale_chart_and_readme_without_writing(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp);(root / '.github').mkdir()
            (root / '.github' / 'idle-benchmark.json').write_text(json.dumps(fixture()))
            (root / 'README.md').write_text(chart.START + '\nstale\n' + chart.END)
            self.assertEqual(chart.main([], root), 0)
            self.assertEqual(chart.main(['--check'], root), 0)
            picture = root / '.github' / 'idle-benchmark.svg'
            picture.write_text('stale')
            self.assertEqual(chart.main(['--check'], root), 1)
            self.assertEqual(picture.read_text(), 'stale')
            self.assertEqual(chart.main([], root), 0)
            (root / 'README.md').write_text(chart.START + '\nstale\n' + chart.END)
            self.assertEqual(chart.main(['--check'], root), 1)


if __name__ == '__main__':
    unittest.main()
