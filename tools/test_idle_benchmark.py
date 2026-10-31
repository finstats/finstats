"""Invented cgroup counters only; never contacts Docker or SSH."""
import base64
import json
import os
import pathlib
import subprocess
import sys
import io
import re
import shlex
import shutil
import tempfile
import uuid
from unittest import mock
import unittest

import idle_benchmark as bench


def endpoint(index, usage=0, status="ok", identity="a", members=("app", "db")):
    return {"type": "sample", "sample": index, "monotonic_s": index * 10.0,
            "utc": "2026-10-31T12:00:00Z", "containers": [
                {"name": name, "status": status, "identity": [identity, name],
                 "cpu_usec": usage, "cpu_monotonic_s": index * 10.0, "memory_bytes": 40, "working_set_bytes": 30,
                 "anon_bytes": 20} for name in members]}


class IntervalTests(unittest.TestCase):
    def test_baseline_and_two_container_total_use_one_core(self):
        rows = bench.measure(endpoint(1, 500_000), endpoint(0))
        self.assertEqual(rows[-1]["cpu_pct_one_core"], 10.0)
        self.assertEqual(rows[-1]["cpu_seconds"], 1.0)
        self.assertEqual(rows[-1]["working_set_bytes"], 60)
        self.assertIsNone(bench.measure(endpoint(0), None)[-1]["cpu_pct_one_core"])

    def test_cpu_uses_counter_read_times_after_inspect_not_schedule_times(self):
        before, after = endpoint(0), endpoint(1, 1_000_000)
        before["containers"][0]["cpu_monotonic_s"] = 2
        after["containers"][0]["cpu_monotonic_s"] = 22
        after["containers"][1]["cpu_monotonic_s"] = 10
        rows = bench.measure(after, before)
        self.assertEqual(rows[0]["cpu_pct_one_core"], 5)
        self.assertEqual(rows[1]["cpu_pct_one_core"], 10)
        self.assertAlmostEqual(rows[-1]["cpu_pct_one_core"], 2 / 15 * 100)
        self.assertEqual(rows[-1]["interval_seconds"], 15)

    def test_nonoverlapping_member_windows_preserve_member_rates_but_refuse_total(self):
        before, after = endpoint(0), endpoint(1, 500000)
        before["containers"][1]["cpu_monotonic_s"] = 15
        after["containers"][1]["cpu_monotonic_s"] = 25
        rows = bench.measure(after, before)
        self.assertTrue(all(row["cpu_seconds"] is not None for row in rows[:-1]))
        self.assertIsNone(rows[-1]["cpu_seconds"])
        self.assertEqual(rows[-1]["cpu_issue"], "member CPU windows do not overlap")

    def test_restart_down_and_counter_reset_never_bridge_intervals(self):
        for current, previous in [(endpoint(1, identity="b"), endpoint(0)),
                                  (endpoint(1, status="down"), endpoint(0)),
                                  (endpoint(1, 0), endpoint(0, 20)),
                                  (endpoint(2, 50), endpoint(1, status="unreadable"))]:
            with self.subTest(current=current):
                row = bench.measure(current, previous)[-1]
                self.assertIsNone(row["cpu_seconds"])
                self.assertIsNotNone(row["cpu_issue"])

    def test_missing_member_never_becomes_a_smaller_stack(self):
        current = endpoint(1, 100)
        current["containers"][1] = {"name": "db", "status": "unreadable"}
        row = bench.measure(current, endpoint(0))[-1]
        self.assertIsNone(row["working_set_bytes"])
        self.assertIsNone(row["cpu_seconds"])

    def test_two_names_for_same_container_never_double_count_a_stack(self):
        current, previous = endpoint(1, 100), endpoint(0)
        current["containers"][1]["identity"] = current["containers"][0]["identity"]
        row = bench.measure(current, previous)[-1]
        self.assertEqual(row["status"], "partial")
        self.assertIsNone(row["working_set_bytes"])
        self.assertIsNone(row["cpu_seconds"])

    def test_summary_marks_partial_cpu_as_lower_bound_and_weights_time(self):
        samples = [endpoint(0), endpoint(1, 500_000), endpoint(2, status="down"),
                   endpoint(3, 2_000_000), endpoint(4, 3_000_000)]
        rows = []
        for i, sample in enumerate(samples):
            rows.extend(bench.measure(sample, samples[i - 1] if i else None))
        result = bench.summarize(rows, expected_intervals=4, expected_endpoints=5)["total"]
        self.assertEqual(result["valid_intervals"], 2)
        self.assertEqual(result["covered_seconds"], 20)
        self.assertEqual(result["cpu_seconds"], 3)
        self.assertEqual(result["cpu_mean_pct_one_core"], 15)
        self.assertTrue(result["cpu_seconds_lower_bound"])
        self.assertEqual(result["memory_samples"], 4)


class TargetTests(unittest.TestCase):
    def test_cli_groups_local_and_remote_containers_and_options(self):
        args, targets = bench.parse_args([
            "--target", "FinStats=local", "--container", "FinStats=finstats",
            "--version", "FinStats=2.3.2", "--target", "Streamystats=ssh:media-alias",
            "--container", "Streamystats=stream-app", "--container", "Streamystats=stream-db",
            "--ssh-option", "Streamystats=IdentityFile=/tmp/key with spaces", "--output", "/tmp/result"])
        self.assertEqual((args.duration, args.interval), (300, 10))
        self.assertEqual(targets[1]["containers"], ["stream-app", "stream-db"])
        command = bench.agent_command(targets[1], {"duration": 300, "interval": 10, "timeout": 5})
        self.assertEqual(command[:3], ["ssh", "-T", "-o"])
        self.assertIn("IdentityFile=/tmp/key with spaces", command)
        remote = shlex.split(command[-1])
        self.assertEqual(remote[:3], ["python3", "-u", "-c"])
        self.assertIn("def sample_container", base64.b64decode(re.search(r'b64decode\("([A-Za-z0-9+/=]+)"\)', remote[3])[1]).decode())
        self.assertEqual(json.loads(base64.b64decode(remote[4]))["containers"], ["stream-app", "stream-db"])

    def test_remote_bootstrap_contains_no_source_backslashes_or_newlines_for_fish(self):
        target = {"name":"Remote", "ssh":"fixture", "containers":["app"], "ssh_options":[]}
        command = bench.agent_command(target, {"duration":10, "interval":10, "timeout":5})
        bootstrap = shlex.split(command[-1])[3]
        self.assertNotIn("\\", bootstrap)
        self.assertNotIn("\n", bootstrap)

    def test_rejects_ambiguous_or_unsafe_targets_before_launch(self):
        base = ["--output", "/tmp/result"]
        bad = [ ["--target", "x=ssh:-oProxyCommand=bad", "--container", "x=app"],
                ["--target", "x=local", "--container", "x=--help"],
                ["--target", "x=local", "--container", "x=app", "--container", "x=app"],
                ["--target", "x=local", "--container", "x=total"],
                ["--target", "x=local"],
                ["--target", "x=local", "--container", "x=app", "--duration", "11", "--interval", "10"],
                ["--target", "x=local", "--container", "x=app", "--interval", "nan"]]
        for argv in bad:
            with self.subTest(argv=argv), mock.patch("sys.stderr"), self.assertRaises(SystemExit):
                bench.parse_args(base + argv)

    def test_json_config_uses_same_validation_and_preserves_notes(self):
        with tempfile.TemporaryDirectory() as temp:
            config = pathlib.Path(temp) / "targets.json"
            config.write_text(json.dumps({"targets": [
                {"name": "FinStats", "containers": ["app"], "version": "2.3.2",
                 "notes": "No browser open; background jobs enabled"}]}))
            _, targets = bench.parse_args(["--config", str(config), "--output", temp + "/out"])
            self.assertEqual(targets[0]["notes"], "No browser open; background jobs enabled")


class HostAgentTests(unittest.TestCase):
    def test_cgroup_memory_and_identity_come_from_host_files(self):
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            proc = root / "proc" / "123"
            proc.mkdir(parents=True)
            (proc / "cgroup").write_text("0::/docker/" + "a" * 64 + "\n")
            (proc / "status").write_text("Name: app\nVmRSS: 12 kB\n")
            group = root / "sys" / "docker" / ("a" * 64)
            group.mkdir(parents=True)
            (group / "cpu.stat").write_text("usage_usec 400000\n")
            (group / "memory.current").write_text("30000\n")
            (group / "memory.stat").write_text("anon 16000\ninactive_file 8000\n")
            (group / "cpu.max").write_text("max 100000\n")
            (group / "cpuset.cpus.effective").write_text("0-3\n")
            info = {"id": "a" * 64, "pid": 123, "started_at": "2026-10-31T12:00:00Z",
                    "restart_count": 0, "running": True, "image": "sha256:abc", "image_ref": "example:1"}
            with mock.patch.object(bench, "inspect_container", return_value=info):
                row = bench.sample_container("app", 5, root / "proc", root / "sys")
            self.assertEqual(row["status"], "ok")
            self.assertEqual(row["working_set_bytes"], 22000)
            self.assertEqual(row["anon_bytes"], 16000)
            self.assertEqual(row["rss_bytes"], 12288)
            (group / "memory.stat").write_text("anon 16000\n")
            with mock.patch.object(bench, "inspect_container", return_value=info):
                row = bench.sample_container("app", 5, root / "proc", root / "sys")
            self.assertEqual(row["status"], "unreadable")
            self.assertNotIn("cpu_usec", row)

    def test_unrelated_cgroup_pid_never_reads_a_host_group_as_container(self):
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            proc = root / "proc" / "123"
            proc.mkdir(parents=True)
            (proc / "cgroup").write_text("0::/user.slice/not-the-container\n")
            info = {"id": "b" * 64, "pid": 123, "running": True}
            with mock.patch.object(bench, "inspect_container", return_value=info), mock.patch.object(bench, "counters") as read:
                row = bench.sample_container("app", 5, root / "proc", root / "sys")
            self.assertEqual(row["status"], "unreadable")
            self.assertIn("identity", row["error"])
            read.assert_not_called()

    def test_systemd_full_identity_works_and_short_or_different_ids_fail(self):
        for identity, cgroup_id, expected in [("a" * 64, "a" * 64, "ok"),
                                               ("a" * 12, "a" * 12, "unreadable"),
                                               ("b" * 64, "a" * 64, "unreadable")]:
            with self.subTest(identity=identity), tempfile.TemporaryDirectory() as temp:
                root = pathlib.Path(temp)
                proc = root / "proc" / "123"
                proc.mkdir(parents=True)
                path = "/system.slice/docker-" + cgroup_id + ".scope"
                (proc / "cgroup").write_text("0::" + path + "\n")
                group = root / "sys" / path.lstrip("/")
                group.mkdir(parents=True)
                (group / "cpu.stat").write_text("usage_usec 1\n")
                (group / "memory.stat").write_text("anon 1\ninactive_file 1\n")
                (group / "memory.current").write_text("2\n")
                info = {"id": identity, "pid":123, "running":True, "started_at":"fixture", "restart_count":0}
                with mock.patch.object(bench, "inspect_container", return_value=info):
                    row = bench.sample_container("app", 5, root / "proc", root / "sys")
                self.assertEqual(row["status"], expected)

    def test_inspect_emits_allowlisted_metadata_and_sanitized_errors(self):
        with mock.patch.object(bench.subprocess, "run", side_effect=bench.subprocess.TimeoutExpired("secret", 5)):
            with self.assertRaisesRegex(bench.SampleError, "docker inspect timed out"):
                bench.inspect_container("app", 5)
        info = {"id": "abc", "pid": 0, "running": False}
        with mock.patch.object(bench, "inspect_container", return_value=info):
            row = bench.sample_container("app", 5)
        self.assertEqual(row["status"], "down")
        self.assertNotIn("cpu_usec", row)


class AgentLifecycleTests(unittest.TestCase):
    def test_remote_docker_context_is_rejected_before_reading_host_cgroups(self):
        with mock.patch.dict(os.environ, {"DOCKER_HOST": "ssh://invented-host", "DOCKER_CONTEXT": ""}), self.assertRaisesRegex(bench.SampleError, "host-local"):
            bench.require_local_docker(5)
        with mock.patch.dict(os.environ, {"DOCKER_HOST": "unix:///run/user/1000/docker.sock", "DOCKER_CONTEXT": ""}):
            bench.require_local_docker(5)

    def test_docker_context_overrides_local_docker_host(self):
        with mock.patch.dict(os.environ, {"DOCKER_CONTEXT":"remote", "DOCKER_HOST":"unix:///var/run/docker.sock"}), mock.patch.object(bench, "host_command", return_value="ssh://fixture") as command:
            with self.assertRaisesRegex(bench.SampleError, "host-local"):
                bench.require_local_docker(5)
        self.assertEqual(command.call_args.args[0][:4], ["docker", "context", "inspect", "remote"])

    def test_missed_endpoints_are_marked_instead_of_catching_up_as_short_intervals(self):
        stopped = mock.Mock()
        stopped.wait.return_value = False
        emitted = io.StringIO()
        # baseline consumes 25 seconds; endpoint 1 is missed, endpoint 2 can be read.
        with mock.patch.object(bench, "require_local_docker"), mock.patch.object(bench, "host_metadata", return_value={}), mock.patch.object(bench.time, "monotonic", side_effect=[0, 0, 0, 25, 25, 25, 25, 25, 25, 25]), mock.patch.object(bench, "sample_container", return_value=endpoint(0, members=("app",))["containers"][0]) as sample, mock.patch("sys.stdout", new=emitted):
            bench.run_agent({"duration": 20, "interval": 10, "timeout": 5, "containers": ["app"]}, stopped)
        events = [json.loads(line) for line in emitted.getvalue().splitlines()]
        self.assertEqual(events[2]["containers"][0]["status"], "unreadable")
        self.assertEqual(sample.call_count, 2)
        self.assertIn("read_duration_s", events[1])

    def test_agent_exits_when_finished_even_if_controller_stdin_stays_open(self):
        with tempfile.TemporaryDirectory() as temp:
            docker = pathlib.Path(temp) / "docker"
            docker.write_text("#!/usr/bin/env python3\nimport json,sys\nif sys.argv[1]=='context': print('unix:///var/run/docker.sock')\nelif sys.argv[1]=='info': print('{}')\nelse: print(json.dumps({'running':False,'pid':0}))\n")
            docker.chmod(0o700)
            target = {"name":"Fixture", "ssh":None, "containers":["app"], "ssh_options":[]}
            command = bench.agent_command(target, {"duration":0.01, "interval":0.01, "timeout":1})
            process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                       env={**os.environ, "PATH":temp + os.pathsep + os.environ["PATH"]}, start_new_session=True)
            try:
                try:
                    self.assertEqual(process.wait(timeout=2), 0)
                except subprocess.TimeoutExpired:
                    self.fail("finished agent held its interpreter open while stdin remained open")
            finally:
                bench.stop_process(process)

    def test_disconnect_stops_agent_without_waiting_for_next_interval(self):
        stopped = mock.Mock()
        stopped.wait.return_value = True
        with mock.patch.object(bench, "require_local_docker"), mock.patch.object(bench, "host_metadata", return_value={}), mock.patch.object(bench, "sample_container") as sample, mock.patch("sys.stdout", new=io.StringIO()):
            bench.run_agent({"duration": 300, "interval": 10, "timeout": 5, "containers": ["app"]}, stopped)
        sample.assert_not_called()


class ArtifactTests(unittest.TestCase):
    def test_partial_run_writes_private_raw_csv_json_and_honest_markdown(self):
        with tempfile.TemporaryDirectory() as temp:
            args, targets = bench.parse_args(["--target", "FinStats=local", "--container", "FinStats=app",
                "--container", "FinStats=db", "--version", "FinStats=2.3.2", "--duration", "20",
                "--interval", "10", "--output", temp + "/result"])
            events = [{"type": "metadata", "host": {"logical_cpus": 4}}, endpoint(0),
                      endpoint(1, 500000), endpoint(2, status="down")]
            source = "import sys; sys.stdout.write(" + repr("".join(json.dumps(e) + "\n" for e in events)) + ")"
            with mock.patch.object(bench, "agent_command", return_value=[sys.executable, "-c", source]), mock.patch("sys.stdout", new=io.StringIO()):
                self.assertEqual(bench.run(args, targets), 1)
            output = args.output
            summary = json.loads((output / "summary.json").read_text())
            total = summary["targets"][0]["metrics"]["total"]
            self.assertEqual(total["valid_intervals"], 1)
            self.assertTrue(total["cpu_seconds_lower_bound"])
            self.assertIn("≥ 1.000", (output / "comparison.md").read_text())
            self.assertIn("1/2", (output / "comparison.md").read_text())
            self.assertEqual(len((output / "samples.jsonl").read_text().splitlines()), 4)
            self.assertEqual(len((output / "samples.csv").read_text().splitlines()), 10)
            self.assertEqual(output.stat().st_mode & 0o777, 0o700)
            for path in output.iterdir():
                self.assertEqual(path.stat().st_mode & 0o777, 0o600)
            with self.assertRaises(FileExistsError):
                bench.run(args, targets)

    def test_failed_target_has_no_zero_usage_claim_and_does_not_hide_good_target(self):
        with tempfile.TemporaryDirectory() as temp:
            args, targets = bench.parse_args(["--target", "Good=local", "--container", "Good=app",
                "--target", "Failed=ssh:example", "--container", "Failed=db", "--duration", "10",
                "--interval", "10", "--output", temp + "/result"])
            events = [endpoint(0, members=("app",)), endpoint(1, 100000, members=("app",))]
            good = "print(" + repr("\n".join(json.dumps(e) for e in events)) + ")"
            def command(target, timing):
                return [sys.executable, "-c", good if target["name"] == "Good" else "raise SystemExit(3)"]
            with mock.patch.object(bench, "agent_command", side_effect=command), mock.patch("sys.stdout", new=io.StringIO()):
                self.assertEqual(bench.run(args, targets), 1)
            summary = json.loads((args.output / "summary.json").read_text())
            self.assertEqual(summary["targets"][0]["metrics"]["total"]["valid_intervals"], 1)
            self.assertIsNone(summary["targets"][1]["metrics"]["total"]["cpu_seconds"])
            self.assertIn("n/a", (args.output / "comparison.md").read_text())

    def test_nonfinite_json_and_safe_agent_errors_preserve_partial_artifacts(self):
        for payload, expected in [('{"type":"metadata","host":{"cpus":NaN}}', "invalid agent output"),
                                  ('{"type":"metadata","host":{"cpus":1e999}}', "invalid agent output"),
                                  (json.dumps({"type":"error", "error":{"hostile":"body"}}), "could not collect"),
                                  (json.dumps({"type":"error", "error":"Docker daemon must be host-local through a Unix socket"}), "host-local")]:
            with self.subTest(payload=payload), tempfile.TemporaryDirectory() as temp:
                args, targets = bench.parse_args(["--target", "Failed=local", "--container", "Failed=app", "--duration", "10", "--interval", "10", "--output", temp + "/out"])
                source = "print(" + repr(payload) + ")"
                with mock.patch.object(bench, "agent_command", return_value=[sys.executable, "-c", source]), mock.patch("sys.stdout", new=io.StringIO()):
                    self.assertEqual(bench.run(args, targets), 1)
                summary = json.loads((args.output / "summary.json").read_text())
                self.assertIn(expected, " ".join(summary["targets"][0]["errors"]))
                self.assertTrue((args.output / "comparison.md").exists())

    def test_fake_ssh_executes_remote_source_without_installed_script(self):
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            ssh = root / "ssh"
            ssh.write_text("#!/usr/bin/env python3\nimport subprocess,sys,shutil\nsys.exit(subprocess.run(sys.argv[-1], shell=True, executable=shutil.which('fish') or '/bin/sh').returncode)\n")
            ssh.chmod(0o700)
            args, targets = bench.parse_args(["--target", "Remote=ssh:invented-alias", "--container", "Remote=app",
                "--duration", "10", "--interval", "10", "--output", temp + "/result"])
            events = [endpoint(0, members=("app",)), endpoint(1, 100000, members=("app",))]
            # Includes shell metacharacters to exercise OpenSSH's shell boundary.
            payload = "import json; marker=\"apostrophe ' dollar $ backtick `\"; print(" + repr("\n".join(json.dumps(e) for e in events)) + ")"
            original = bench.agent_command
            def command(target, timing):
                with mock.patch.object(bench.Path, "read_bytes", return_value=payload.encode()):
                    return original(target, timing)
            with mock.patch.dict(os.environ, {"PATH": temp + os.pathsep + os.environ["PATH"]}), mock.patch.object(bench, "agent_command", side_effect=command), mock.patch("sys.stdout", new=io.StringIO()):
                self.assertEqual(bench.run(args, targets), 0)
            summary = json.loads((args.output / "summary.json").read_text())
            self.assertEqual(summary["targets"][0]["metrics"]["total"]["cpu_mean_pct_one_core"], 1)


@unittest.skipUnless(os.environ.get("FINSTATS_BENCHMARK_DOCKER_TESTS") == "1", "opt-in cached Alpine Docker fixture")
class DockerMetadataIntegrationTests(unittest.TestCase):
    def test_container_without_healthcheck_or_image_labels_is_inspectable(self):
        for with_label in (True, False):
            name = "finstats-bench-test-" + uuid.uuid4().hex
            command = ["docker", "create", "--pull=never", "--network", "none", "--name", name]
            if with_label:
                command.extend(["--label", "no.finstats.benchmark-test=" + name])
            command.append("alpine:3.22")
            identity = subprocess.check_output(command, text=True).strip()
            try:
                row = bench.inspect_container(identity, 5)
                self.assertFalse(row["running"])
                self.assertIsNone(row["health"])
                self.assertIsNone(row["version_label"])
            finally:
                subprocess.run(["docker", "rm", identity], stdout=subprocess.DEVNULL, check=True)


if __name__ == "__main__":
    unittest.main()
