#!/usr/bin/env python3
"""Observe Docker cgroup v2 counters locally or over SSH, without app requests.

Run with --help. Detailed usage and limitations live in the FinStats contributing guide.
Only the Python standard library is required (Python 3.9 or newer).
"""
import argparse
import base64
import csv
import json
import math
import os
from pathlib import Path
import platform
import queue
import re
import shlex
import signal
import statistics
import subprocess
import sys
import time
import threading

MEMORY_FIELDS = ("memory_bytes", "working_set_bytes", "anon_bytes")


def measure(current, previous):
    """Derive adjacent endpoint deltas; a stack needs every configured member."""
    before = {c["name"]: c for c in previous["containers"]} if previous else {}
    elapsed = current["monotonic_s"] - previous["monotonic_s"] if previous else None
    rows = []
    for member in current["containers"]:
        row = {"sample": current["sample"], "utc": current["utc"],
               "monotonic_s": current["monotonic_s"], "container": member["name"],
               "status": member["status"], "interval_seconds": elapsed,
               "cpu_seconds": None, "cpu_pct_one_core": None, "cpu_issue": None}
        readable = member["status"] == "ok"
        for key in MEMORY_FIELDS:
            row[key] = member[key] if readable else None
        old = before.get(member["name"])
        read_time = member.get("cpu_monotonic_s", current["monotonic_s"])
        row["monotonic_s"] = read_time
        if readable and old and old["status"] == "ok":
            row["interval_seconds"] = read_time - old["cpu_monotonic_s"]
        if not previous:
            row["cpu_issue"] = "baseline"
        elif not readable or not old or old["status"] != "ok":
            row["cpu_issue"] = "unreadable endpoint"
        elif member["identity"] != old["identity"]:
            row["cpu_issue"] = "container restarted or replaced"
        elif row["interval_seconds"] <= 0 or member["cpu_usec"] < old["cpu_usec"]:
            row["cpu_issue"] = "clock or counter reset"
        else:
            row["cpu_seconds"] = (member["cpu_usec"] - old["cpu_usec"]) / 1_000_000
            row["cpu_pct_one_core"] = row["cpu_seconds"] / row["interval_seconds"] * 100
        rows.append(row)
    total = {"sample": current["sample"], "utc": current["utc"],
             "monotonic_s": current["monotonic_s"], "container": "total",
             "status": "ok" if all(r["status"] == "ok" for r in rows) else "partial",
             "interval_seconds": elapsed}
    for key in MEMORY_FIELDS + ("cpu_seconds", "cpu_pct_one_core"):
        total[key] = sum(r[key] for r in rows) if rows and all(r[key] is not None for r in rows) else None
    total["cpu_issue"] = next((r["cpu_issue"] for r in rows if r["cpu_issue"]), None)
    if total["cpu_seconds"] is not None:
        # Group counters share a representative endpoint: the mean of the member
        # counter-read timestamps. This keeps CPU seconds and coverage consistent.
        total["monotonic_s"] = statistics.mean(r["monotonic_s"] for r in rows)
        total["interval_seconds"] = statistics.mean(r["interval_seconds"] for r in rows)
        total["cpu_pct_one_core"] = total["cpu_seconds"] / total["interval_seconds"] * 100
        if max(c["cpu_monotonic_s"] for c in previous["containers"]) >= min(c["cpu_monotonic_s"] for c in current["containers"]):
            total["cpu_seconds"] = total["cpu_pct_one_core"] = None
            total["cpu_issue"] = "member CPU windows do not overlap"
    identities = [c["identity"] for c in current["containers"] if c["status"] == "ok"]
    if len({json.dumps(identity, sort_keys=True) for identity in identities}) != len(identities):
        total["status"] = "partial"
        total["cpu_issue"] = "duplicate container identity"
        for key in MEMORY_FIELDS + ("cpu_seconds", "cpu_pct_one_core"):
            total[key] = None
    return rows + [total]


def nearest_rank(values, percentile):
    """Nearest rank: ceil(p * n), one based; no interpolation or invented samples."""
    return sorted(values)[math.ceil(percentile * len(values)) - 1] if values else None


def summarize(rows, expected_intervals, expected_endpoints, container_names=None):
    result = {}
    for name in (container_names or dict.fromkeys(r["container"] for r in rows)):
        selected = [r for r in rows if r["container"] == name]
        valid = [r for r in selected if r["cpu_seconds"] is not None]
        memory = [r for r in selected if r["working_set_bytes"] is not None]
        covered = sum(r["interval_seconds"] for r in valid)
        cpu = sum(r["cpu_seconds"] for r in valid)
        result[name] = {
            "valid_intervals": len(valid), "expected_intervals": expected_intervals,
            "covered_seconds": covered, "cpu_seconds": cpu if valid else None,
            "cpu_seconds_lower_bound": len(valid) != expected_intervals,
            "cpu_mean_pct_one_core": cpu / covered * 100 if covered else None,
            "cpu_p90_pct_one_core": nearest_rank([r["cpu_pct_one_core"] for r in valid], 0.9),
            "cpu_median_pct_one_core": statistics.median(r["cpu_pct_one_core"] for r in valid) if valid else None,
            "cpu_peak_pct_one_core": max((r["cpu_pct_one_core"] for r in valid), default=None),
            "memory_samples": len(memory), "expected_endpoints": expected_endpoints,
            "working_set_mean_bytes": statistics.mean(r["working_set_bytes"] for r in memory) if memory else None,
            "working_set_p90_bytes": nearest_rank([r["working_set_bytes"] for r in memory], 0.9),
            "working_set_min_bytes": min((r["working_set_bytes"] for r in memory), default=None),
            "working_set_peak_bytes": max((r["working_set_bytes"] for r in memory), default=None),
            "anon_mean_bytes": statistics.mean(r["anon_bytes"] for r in memory) if memory else None,
        }
    return result


class SampleError(Exception):
    pass


SAFE_AGENT_ERRORS = {
    "invalid agent output", "Docker daemon must be host-local through a Unix socket",
    "docker context timed out", "docker context unavailable",
    "docker context failed (check host permissions and container name)",
}


def strict_json(text):
    def reject_constant(value):
        raise ValueError("non-finite JSON value")
    def finite_float(value):
        number = float(value)
        if not math.isfinite(number):
            raise ValueError("non-finite JSON value")
        return number
    return json.loads(text, parse_constant=reject_constant, parse_float=finite_float)


def parse_args(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="Example: --target FinStats=local --container FinStats=finstats --version FinStats=2.3.2\n"
               "Remote: --target Streamystats=ssh:media --container Streamystats=app --container Streamystats=db\n"
               "SSH uses config aliases; repeat --ssh-option Streamystats=IdentityFile=/path/to/key.\n"
               "Only observe an already idle deployment. No application requests or container changes are made.")
    parser.add_argument("--config", type=Path, help="JSON object with a targets array (see contributing guide)")
    for flag in ("target", "container", "version", "ssh-option", "note", "program"):
        parser.add_argument("--" + flag, action="append", default=[], metavar="NAME=VALUE")
    parser.add_argument("--scenario", choices=("idle", "playback"), default="idle", help="operator-declared workload context; never starts or verifies playback")
    parser.add_argument("--duration", type=float, default=300, help="window seconds, default 300")
    parser.add_argument("--interval", type=float, default=10, help="endpoint spacing seconds, default 10")
    parser.add_argument("--timeout", type=float, default=5, help="each host command timeout seconds, default 5")
    parser.add_argument("--output", type=Path, required=True, help="new private result directory (must not exist)")
    args = parser.parse_args(argv)
    try:
        for value in (args.duration, args.interval, args.timeout):
            if not math.isfinite(value) or value <= 0:
                raise ValueError("duration, interval and timeout must be finite and positive")
        ratio = args.duration / args.interval
        if not math.isfinite(ratio) or not math.isclose(ratio, round(ratio), abs_tol=1e-9) or not 1 <= ratio <= 100_000:
            raise ValueError("duration must be an exact multiple of interval (1 to 100000 intervals)")
        targets = json.loads(args.config.read_text())["targets"] if args.config else []
        if not isinstance(targets, list):
            raise ValueError("config targets must be an array")
        def pair(value):
            name, sep, text = value.partition("=")
            if not sep or not name or not text:
                raise ValueError("target options must be NAME=VALUE")
            return name, text
        for value in args.target:
            name, transport = pair(value)
            if transport != "local" and not transport.startswith("ssh:"):
                raise ValueError("target transport must be local or ssh:ALIAS")
            targets.append({"name": name, "ssh": transport[4:] if transport.startswith("ssh:") else None,
                            "containers": []})
        names = [t["name"] for t in targets]
        if not names or len(names) != len(set(names)):
            raise ValueError("supply at least one target with a unique name")
        for field, values in (("containers", args.container), ("version", args.version),
                              ("ssh_options", args.ssh_option), ("notes", args.note), ("program", args.program)):
            for value in values:
                name, text = pair(value)
                target = next((t for t in targets if t["name"] == name), None)
                if target is None:
                    raise ValueError("option names an unknown target")
                if field in ("containers", "ssh_options"):
                    target.setdefault(field, []).append(text)
                else:
                    target[field] = text
        for target in targets:
            if set(target) - {"name", "ssh", "containers", "version", "ssh_options", "notes", "program"}:
                raise ValueError("unknown target config field")
            if not isinstance(target["name"], str) or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_. -]{0,79}", target["name"]):
                raise ValueError("target names must be short plain labels")
            target.setdefault("ssh", None)
            target.setdefault("ssh_options", [])
            target.setdefault("version", "unspecified")
            target.setdefault("notes", "")
            target.setdefault("program", "auto")
            if target["program"] not in ("auto", "none", "finstats", "jellystat", "streamystats"):
                raise ValueError("program must be auto, none, finstats, jellystat or streamystats")
            host = target["ssh"]
            if host is not None and (not isinstance(host, str) or not re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.@:-]{0,253}", host)):
                raise ValueError("SSH target must be a config alias or user@host, without options or shell text")
            members = target["containers"]
            if not isinstance(members, list) or not members or len(members) != len(set(members)):
                raise ValueError("each target needs distinct container names")
            for member in members:
                if not isinstance(member, str) or member == "total" or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}", member):
                    raise ValueError("invalid container name (total is reserved)")
            if not isinstance(target["ssh_options"], list) or (target["ssh_options"] and not host):
                raise ValueError("ssh_options requires an SSH target and an array")
            for option in target["ssh_options"]:
                if not isinstance(option, str) or not re.fullmatch(r"[A-Za-z][A-Za-z0-9]*=[^\x00-\x1f\x7f]+", option):
                    raise ValueError("SSH options must be Key=Value without control characters")
            for field in ("version", "notes"):
                if not isinstance(target[field], str) or any(ord(c) < 32 or ord(c) == 127 for c in target[field]):
                    raise ValueError("version and notes must be text without control characters")
    except (ValueError, KeyError, TypeError, OSError) as exc:
        # Do not repeat config values, paths or upstream output in errors.
        parser.error(str(exc) if isinstance(exc, ValueError) and not isinstance(exc, json.JSONDecodeError) else "invalid or unreadable target config")
    return args, targets


def agent_command(target, timing):
    config = base64.b64encode(json.dumps({**timing, "containers": target["containers"], "program": target.get("program", "auto")}).encode()).decode()
    if target["ssh"] is None:
        return [sys.executable, "-u", str(Path(__file__).resolve()), "--agent", config]
    command = ["ssh", "-T", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10"]
    for option in target["ssh_options"]:
        command.extend(["-o", option])
    # OpenSSH invokes the login shell. Fish processes backslashes even within
    # single quotes, so send source as base64 rather than shell-quoting Python text.
    source = base64.b64encode(Path(__file__).read_bytes()).decode()
    bootstrap = 'import base64;exec(compile(base64.b64decode("' + source + '"),"<idle-benchmark>","exec"))'
    remote = "python3 -u -c " + shlex.quote(bootstrap) + " " + shlex.quote(config)
    return command + ["--", target["ssh"], remote]


INSPECT_TEMPLATE = '{' + ','.join('"' + key + '":{{json ' + field + '}}' for key, field in (
    ("id", ".Id"), ("image", ".Image"), ("image_ref", ".Config.Image"),
    ("pid", ".State.Pid"), ("running", ".State.Running"), ("started_at", ".State.StartedAt"),
    ("restart_count", ".RestartCount"), ("health", ".State.Health.Status"),
    ("version_label", '(index .Config.Labels "org.opencontainers.image.version")'),
    ("revision_label", '(index .Config.Labels "org.opencontainers.image.revision")'))) + '}'
# Docker uses strict missing-key lookups, including inside if/with expressions.
# Optional health and label maps need index before any nested field access.
INSPECT_TEMPLATE = INSPECT_TEMPLATE.replace('{{json .State.Health.Status}}', '{{with (index .State "Health")}}{{json (index . "Status")}}{{else}}null{{end}}')
for _label in ("org.opencontainers.image.version", "org.opencontainers.image.revision"):
    INSPECT_TEMPLATE = INSPECT_TEMPLATE.replace('{{json (index .Config.Labels "' + _label + '")}}',
        '{{with (index .Config "Labels")}}{{json (index . "' + _label + '")}}{{else}}null{{end}}')


def host_command(command, timeout, label):
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=timeout, check=False)
    except subprocess.TimeoutExpired:
        raise SampleError(label + " timed out") from None
    except OSError:
        raise SampleError(label + " unavailable") from None
    if result.returncode:
        raise SampleError(label + " failed (check host permissions and container name)")
    return result.stdout


def inspect_container(name, timeout):
    try:
        return json.loads(host_command(["docker", "inspect", "--type", "container", "--format", INSPECT_TEMPLATE, name], timeout, "docker inspect"))
    except (ValueError, TypeError):
        raise SampleError("invalid docker inspect response") from None


def infer_program(image):
    image = image if isinstance(image, str) else ""
    name = image.lower().split("@", 1)[0].rsplit("/", 1)[-1].split(":", 1)[0]
    if name == "finstats": return "finstats"
    if name == "jellystat": return "jellystat"
    if name == "streamystats-nextjs": return "streamystats"
    if name == "streamystats-job-server": return "streamystats-worker"
    if "postgres" in name or "vectorchord" in name: return "postgres"
    return "unknown"


def discover_program_versions(containers, program, timeout):
    """Fixed read-only probes, before the CPU baseline; never record raw output."""
    if program == "none": return []
    results = []
    for index, name in enumerate(containers):
        item = {"container": name, "kind": "unknown", "version": None, "provenance_kind": "unavailable"}
        try:
            info = inspect_container(name, timeout)
            item.update({"image": info.get("image"), "image_ref": info.get("image_ref")})
            kind = program if index == 0 and program != "auto" else infer_program(info.get("image_ref", ""))
            item["kind"] = kind
            if kind == "finstats":
                command, provenance, source = ["finstats", "--version"], "runtime_cli", "finstats --version"
            elif kind == "postgres":
                command, provenance, source = ["postgres", "--version"], "database_cli", "postgres --version"
            elif kind in ("jellystat", "streamystats"):
                source = "/app/package.json" if kind == "jellystat" else "/app/apps/nextjs-app/package.json"
                command, provenance = ["head", "-c", "131073", source], "application_package"
            else:
                item["source"] = "no fixed read-only version source available"
                results.append(item)
                continue
            output = host_command(["docker", "exec", name, *command], timeout, "program version probe")
            if len(output.encode()) > 131072: raise ValueError()
            if provenance == "application_package":
                package = strict_json(output)
                version = package.get("version") if isinstance(package, dict) else None
            else:
                pattern = r"(?i)^finstats\s+([0-9][A-Za-z0-9.+-]*)" if kind == "finstats" else r"^postgres \(PostgreSQL\) ([0-9][A-Za-z0-9.+-]*)"
                match = re.match(pattern, output.strip())
                version = match[1] if match else None
            if not isinstance(version, str) or not re.fullmatch(r"[0-9]+(?:\.[0-9]+){1,3}(?:[-+][A-Za-z0-9.-]+)?", version): raise ValueError()
            item.update({"version": version, "provenance_kind": provenance, "source": source})
        except (SampleError, ValueError, TypeError):
            item["error"] = "fixed version source unavailable or invalid"
        results.append(item)
    return results


def validated_program_versions(value, containers):
    if not isinstance(value, list): raise ValueError("invalid program metadata")
    required = {"container", "kind", "version", "provenance_kind"}
    allowed = required | {"image", "image_ref", "source", "error"}
    kinds = {"finstats", "jellystat", "streamystats", "streamystats-worker", "postgres", "unknown"}
    provenances = {"runtime_cli", "application_package", "database_cli", "unavailable"}
    seen, result = set(), []
    for item in value:
        if not isinstance(item, dict) or not required <= set(item) or set(item) - allowed: raise ValueError("invalid program metadata")
        name, kind, version, provenance = (item[key] for key in ("container", "kind", "version", "provenance_kind"))
        if not isinstance(name, str) or name not in containers or name in seen or not isinstance(kind, str) or kind not in kinds or not isinstance(provenance, str) or provenance not in provenances: raise ValueError("invalid program metadata")
        if version is not None and (not isinstance(version, str) or not re.fullmatch(r"[0-9]+(?:\.[0-9]+){1,3}(?:[-+][A-Za-z0-9.-]+)?", version)): raise ValueError("invalid program metadata")
        if (version is None) != (provenance == "unavailable"): raise ValueError("invalid program metadata")
        for key in set(item) - required:
            text = item[key]
            if text is not None and (not isinstance(text, str) or len(text) > 512 or any(ord(c) < 32 or ord(c) == 127 for c in text)): raise ValueError("invalid program metadata")
        seen.add(name);result.append(dict(item))
    return result


def collection_timeout(duration, timeout, members, remote, program):
    startup = (2 + (0 if program == "none" else 2 * members)) * timeout + (10 if remote else 0)
    return duration + startup + max(30, timeout * (members + 2) * 2)


def counters(path):
    return {key: int(value) for key, value in (line.split() for line in path.read_text().splitlines())}


def sample_container(name, timeout, proc_root=Path("/proc"), cgroup_root=Path("/sys/fs/cgroup")):
    row = {"name": name, "status": "unreadable"}
    try:
        info = inspect_container(name, timeout)
        row["metadata"] = info
        if not info["running"] or info["pid"] <= 0:
            row["status"] = "down"
            return row
        proc = proc_root / str(info["pid"])
        unified = [line[3:] for line in (proc / "cgroup").read_text().splitlines() if line.startswith("0::")]
        if len(unified) != 1 or unified[0] == "/":
            raise SampleError("container has no distinct host cgroup v2 path")
        identity = info["id"]
        parts = Path(unified[0]).parts
        if not re.fullmatch(r"[a-f0-9]{64}", identity) or not any(part in (identity, "docker-" + identity + ".scope") for part in parts):
            raise SampleError("host cgroup does not match container identity")
        group = (cgroup_root / unified[0].lstrip("/")).resolve()
        if cgroup_root.resolve() not in group.parents:
            raise SampleError("container cgroup path escapes cgroup root")
        cpu = counters(group / "cpu.stat")["usage_usec"]
        cpu_time = time.monotonic()
        memory = int((group / "memory.current").read_text())
        stat = counters(group / "memory.stat")
        inactive, anon = stat["inactive_file"], stat["anon"]
        if min(cpu, memory, inactive, anon) < 0:
            raise SampleError("negative cgroup counter")
        identity = [info["id"], info["pid"], info["started_at"], info["restart_count"], group.stat().st_ino]
        data = {"identity": identity, "cgroup": unified[0], "cpu_usec": cpu, "cpu_monotonic_s": cpu_time,
                "memory_bytes": memory, "working_set_bytes": max(0, memory - inactive), "anon_bytes": anon}
        for field, file in (("cpu_max", "cpu.max"), ("cpuset", "cpuset.cpus.effective")):
            try:
                data[field] = (group / file).read_text().strip()
            except OSError:
                data[field] = None
        try:
            data["rss_bytes"] = int(next(line.split()[1] for line in (proc / "status").read_text().splitlines() if line.startswith("VmRSS:"))) * 1024
        except (OSError, ValueError, StopIteration):
            data["rss_bytes"] = None
        row.update(data)
        row["status"] = "ok"
    except SampleError as exc:
        row["error"] = str(exc)
    except (OSError, ValueError, KeyError, TypeError):
        row["error"] = "host cgroup counters unavailable or invalid"
    return row


def utc():
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def host_metadata(timeout):
    result = {"system": platform.system(), "kernel": platform.release(), "machine": platform.machine(),
              "logical_cpus": os.cpu_count(), "python": platform.python_version(), "utc": utc()}
    try:
        cpuinfo = Path("/proc/cpuinfo").read_text()
        result["cpu_model"] = next(line.split(":", 1)[1].strip() for line in cpuinfo.splitlines() if line.startswith("model name"))
    except (OSError, StopIteration):
        result["cpu_model"] = None
    try:
        result["docker"] = json.loads(host_command(["docker", "info", "--format",
            '{"version":{{json .ServerVersion}},"cgroup_version":{{json .CgroupVersion}},"os":{{json .OperatingSystem}},"architecture":{{json .Architecture}}}'], timeout, "docker info"))
    except (SampleError, ValueError):
        result["docker"] = None
    return result


def require_local_docker(timeout):
    context = os.environ.get("DOCKER_CONTEXT")
    endpoint = os.environ.get("DOCKER_HOST") if not context else None
    if not endpoint:
        command = ["docker", "context", "inspect"] + ([context] if context else [])
        endpoint = host_command(command + ["--format", "{{.Endpoints.docker.Host}}"], timeout, "docker context").strip()
    if not endpoint.startswith("unix://"):
        raise SampleError("Docker daemon must be host-local through a Unix socket")


def run_agent(config, stopped=None):
    def emit(event):
        print(json.dumps(event, allow_nan=False), flush=True)
    stopped = stopped or threading.Event()
    require_local_docker(config["timeout"])
    emit({"type": "metadata", "host": host_metadata(config["timeout"]),
          "program_versions": discover_program_versions(config["containers"], config.get("program", "none"), config["timeout"])})
    start = time.monotonic()
    count = round(config["duration"] / config["interval"])
    for index in range(count + 1):
        if stopped.wait(max(0, start + index * config["interval"] - time.monotonic())):
            return
        now = time.monotonic()
        late = now - (start + index * config["interval"])
        if late >= config["interval"]:
            members = [{"name": name, "status": "unreadable", "error": "sampling deadline missed"} for name in config["containers"]]
        else:
            members = [sample_container(name, config["timeout"]) for name in config["containers"]]
        emit({"type": "sample", "sample": index, "utc": utc(), "monotonic_s": now,
              "elapsed_s": now - start, "scheduled_s": index * config["interval"], "lag_s": late,
              "read_duration_s": time.monotonic() - now, "containers": members})


CAVEATS = [
    "Observational idle deployment comparison, not a controlled version regression test. No hardware multiplier is applied.",
    "CPU is percent of one logical core (100% is one core), derived from usage_usec and each container's actual counter-read monotonic timestamp. Stack counters use the mean constituent read timestamp at each endpoint, an estimate across sequential reads; non-overlapping member windows invalidate stack CPU. Raw timestamps show read skew. Mean CPU is time weighted; median and peak describe readable adjacent intervals.",
    "Working set is max(0, memory.current - memory.stat inactive_file); anonymous RAM is memory.stat anon. MiB means 1048576 bytes. Memory statistics include the baseline and all readable endpoints.",
    "Every configured container, including each database and worker, contributes to a stack. A missing member makes that endpoint's stack memory and adjoining CPU intervals unavailable.",
    "CPU seconds with missing intervals are a lower bound. A restart, replacement, counter reset, unreadable endpoint or absent endpoint cannot be bridged. No valid intervals means n/a, not zero.",
    "The sampler sends no application/API traffic and does not restart containers. Existing healthchecks, background jobs, real viewers and other host workloads remain enabled and are not independently instrumented.",
    "Version cells distinguish observed runtime CLI, application-package and database-CLI versions from supplied operator labels or unavailable values. Observations and their image/source provenance are retained; supplied labels remain separate. Package values do not establish immutable release tags, and OCI labels may be stale.",
    "Hosts start independently after connection and metadata setup; inspect counters sequentially within each endpoint. Sampling itself has overhead. See raw UTC windows, elapsed times and read durations; hosts need synchronized clocks to compare UTC.",
    "Hardware, uptime, caches, database size, settings and background work can change results. Include warm-up, activity checks and deployment context in notes before publishing; this tool cannot establish that an application was idle.",
    "Linux host-side cgroup v2 and a local Docker daemon are required. Docker Desktop VM, cgroup v1, remote Docker contexts and inaccessible host cgroups are unsupported. Process RSS, when available, is only the container init process, not a stack memory total.",
]


def markdown(summary):
    def cell(text):
        return str(text).replace("\\", "\\\\").replace("|", "\\|").replace("<", "&lt;").replace(">", "&gt;").replace("\n", " ").replace("\r", " ")
    def number(value, digits=3, divisor=1):
        return "n/a" if value is None else f"{value / divisor:.{digits}f}"
    scenario = summary.get("measurement_scenario", "idle")
    lines = [f"# {scenario.capitalize()} resource comparison", "", "Measured runs only. Review deployment notes and metadata before copying this table into the README.", "",
             "| Deployment | Version (source) | Mean CPU | Median CPU | Peak interval CPU | CPU seconds | Valid intervals | Covered seconds | Mean working set MiB | Working set range MiB | Mean anonymous MiB | Memory endpoints |",
             "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
    for target in summary["targets"]:
        metric = target["metrics"]["total"]
        cpu_seconds = number(metric["cpu_seconds"])
        if metric["cpu_seconds_lower_bound"] and metric["cpu_seconds"] is not None:
            cpu_seconds = "≥ " + cpu_seconds
        percentages = [number(metric[key]) + ("%" if metric[key] is not None else "") for key in
                       ("cpu_mean_pct_one_core", "cpu_median_pct_one_core", "cpu_peak_pct_one_core")]
        memory_range = number(metric["working_set_min_bytes"], 2, 1048576) + " to " + number(metric["working_set_peak_bytes"], 2, 1048576)
        source_label = {"runtime_cli": "runtime CLI", "application_package": "application package", "database_cli": "database CLI", "operator_label": "operator label", "unavailable": "unavailable"}.get(target.get("version_provenance", "operator_label"), "unavailable")
        values = [cell(target["name"]), cell(target["version"]) + " (" + source_label + ")", *percentages, cpu_seconds,
                  f'{metric["valid_intervals"]}/{metric["expected_intervals"]}', number(metric["covered_seconds"]),
                  number(metric["working_set_mean_bytes"], 2, 1048576), memory_range,
                  number(metric["anon_mean_bytes"], 2, 1048576), f'{metric["memory_samples"]}/{metric["expected_endpoints"]}']
        lines.append("| " + " | ".join(values) + " |")
    lines.extend(["", f'Requested window: {summary["timing"]["duration"]:g} seconds; interval: {summary["timing"]["interval"]:g} seconds. Baseline plus interval endpoints. Run status: {summary["status"]}.', ""])
    for target in summary["targets"]:
        lines.append(f'- **{cell(target["name"])}**: containers {", ".join(cell(c) for c in target["containers"])}. Measured UTC: {cell(target.get("first_utc") or "n/a")} to {cell(target.get("last_utc") or "n/a")}. {cell(target["notes"])}')
        if target["errors"]:
            lines.append("  Collection issues: " + "; ".join(cell(e) for e in target["errors"]) + ".")
    lines.extend(["", *["- " + caveat for caveat in summary.get("methodology", CAVEATS)], ""])
    return "\n".join(lines)


def private_file(path):
    return open(path, "x", opener=lambda name, flags: os.open(name, flags, 0o600), encoding="utf-8", newline="")


def stop_process(process):
    # Each child owns a process group. This also reaps local host-command children.
    # Closing SSH stdin/disconnecting lets the remote agent's EOF watcher stop it.
    if process.stdin:
        process.stdin.close()
    if process.poll() is None:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait(timeout=2)
    if process.stdout:
        process.stdout.close()


def run(args, targets):
    args.output.mkdir(mode=0o700, parents=True, exist_ok=False)
    timing = {"duration": args.duration, "interval": args.interval, "timeout": args.timeout}
    expected = round(args.duration / args.interval)
    scenario = getattr(args, "scenario", "idle")
    methodology = list(CAVEATS)
    if scenario == "playback":
        methodology[0] = methodology[0].replace("idle deployment", "playback deployment")
        methodology.append("Playback context is operator-declared: the sampler does not start or verify playback. Record observed activity, stream count and play method before and after the window in notes.")
    summary = {"format_version": 2, "started_utc": utc(), "timing": timing, "status": "complete",
               "measurement_scenario": scenario, "methodology": methodology, "targets": []}
    states = []
    messages = queue.Queue()
    processes = []
    interrupted = False

    def reader(index, process):
        try:
            for line in process.stdout:
                try:
                    event = strict_json(line)
                    if not isinstance(event, dict):
                        raise ValueError()
                except ValueError:
                    messages.put((index, {"type": "error", "error": "invalid agent output"}))
                    break
                messages.put((index, event))
            code = process.wait(timeout=2)
        except (OSError, subprocess.TimeoutExpired):
            code = 1
        messages.put((index, {"type": "done", "returncode": code}))

    with private_file(args.output / "samples.jsonl") as raw:
        try:
            for index, target in enumerate(targets):
                info = {key: target[key] for key in ("name", "version", "containers", "notes")}
                info.update({"transport": "ssh" if target["ssh"] else "local", "errors": [], "host": None,
                             "operator_version_label": info["version"], "version_provenance": "operator_label" if info["version"] != "unspecified" else "unavailable", "program_versions": []})
                summary["targets"].append(info)
                state = {"rows": [], "previous": None, "samples": 0, "done": False,
                         "deadline": time.monotonic() + collection_timeout(args.duration, args.timeout, len(target["containers"]), bool(target["ssh"]), target.get("program", "auto"))}
                states.append(state)
                try:
                    process = subprocess.Popen(agent_command(target, timing), stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                               stderr=subprocess.DEVNULL, text=True, start_new_session=True)
                    processes.append(process)
                    state["process"] = process
                    threading.Thread(target=reader, args=(index, process), daemon=True).start()
                except OSError:
                    info["errors"].append("could not start host agent")
                    state["done"] = True
            while not all(state["done"] for state in states):
                for index, state in enumerate(states):
                    if not state["done"] and time.monotonic() > state["deadline"]:
                        summary["targets"][index]["errors"].append("host agent exceeded collection deadline")
                        state["done"] = True
                        stop_process(state["process"])
                try:
                    index, event = messages.get(timeout=0.2)
                except queue.Empty:
                    continue
                state, info = states[index], summary["targets"][index]
                if state["done"]:
                    continue
                kind = event.get("type")
                if kind == "done":
                    state["done"] = True
                    if event["returncode"]:
                        info["errors"].append("host agent failed; check python3 availability, SSH alias/key/known-host access, Docker and host permissions (SSH diagnostics are suppressed)")
                    continue
                raw.write(json.dumps({"target": info["name"], **event, "measurement_scenario": scenario}, allow_nan=False) + "\n")
                raw.flush()
                if kind == "metadata":
                    info["host"] = event.get("host")
                    try:
                        info["program_versions"] = validated_program_versions(event.get("program_versions", []), info["containers"])
                    except ValueError:
                        info["errors"].append("invalid program version metadata; supplied label retained")
                    main = next((item for item in info["program_versions"] if item["kind"] in ("finstats", "jellystat", "streamystats") and item["version"]), None)
                    if main:
                        info["version"] = main["version"]
                        info["version_provenance"] = main["provenance_kind"]
                elif kind == "sample":
                    if event.get("sample") != state["samples"] or [c.get("name") for c in event.get("containers", [])] != info["containers"]:
                        info["errors"].append("agent endpoint sequence or container membership changed")
                        state["done"] = True
                        continue
                    try:
                        rows = measure(event, state["previous"])
                    except (KeyError, TypeError, ValueError, ZeroDivisionError):
                        info["errors"].append("invalid agent counters")
                        state["done"] = True
                        continue
                    state["rows"].extend({"target": info["name"], **row} for row in rows)
                    state["previous"] = event
                    state["samples"] += 1
                    info.setdefault("first_utc", event["utc"])
                    info["last_utc"] = event["utc"]
                elif kind == "error":
                    error = event.get("error")
                    info["errors"].append(error if isinstance(error, str) and error in SAFE_AGENT_ERRORS else "host agent could not collect counters")
        except KeyboardInterrupt:
            interrupted = True
            summary["status"] = "interrupted"
        finally:
            for process in processes:
                stop_process(process)

    for state, info in zip(states, summary["targets"]):
        info["metrics"] = summarize(state["rows"], expected, expected + 1, info["containers"] + ["total"])
        if state["samples"] != expected + 1:
            info["errors"].append("collection ended before every endpoint was received")
        metric = info["metrics"]["total"]
        if info["errors"] or metric["valid_intervals"] != expected or metric["memory_samples"] != expected + 1:
            if not interrupted:
                summary["status"] = "incomplete"
    summary["finished_utc"] = utc()
    with private_file(args.output / "samples.csv") as handle:
        fields = ["target", "container", "sample", "utc", "monotonic_s", "status", "interval_seconds",
                  "cpu_seconds", "cpu_pct_one_core", "cpu_issue", *MEMORY_FIELDS]
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        for state in states:
            writer.writerows(state["rows"])
    with private_file(args.output / "summary.json") as handle:
        json.dump(summary, handle, indent=2, allow_nan=False)
        handle.write("\n")
    with private_file(args.output / "comparison.md") as handle:
        handle.write(markdown(summary))
    print(f'{scenario.capitalize()} benchmark {summary["status"]}: {args.output}')
    return 130 if interrupted else (0 if summary["status"] == "complete" else 1)


def main():
    # -c on SSH has no script path, so its sole argument is the encoded config.
    if len(sys.argv) == 3 and sys.argv[1] == "--agent":
        config = json.loads(base64.b64decode(sys.argv[2]))
    elif sys.argv[0] == "-c" and len(sys.argv) == 2:
        config = json.loads(base64.b64decode(sys.argv[1]))
    else:
        args, targets = parse_args()
        try:
            return run(args, targets)
        except FileExistsError:
            print("Output directory already exists; choose a new directory.", file=sys.stderr)
            return 2
        except OSError:
            print("Could not write benchmark artifacts; check output directory permissions.", file=sys.stderr)
            return 2
    stopped = threading.Event()
    def watch_stdin():
        try:
            os.read(sys.stdin.fileno(), 1)
        finally:
            stopped.set()
    threading.Thread(target=watch_stdin, daemon=True).start()
    try:
        run_agent(config, stopped)
    except SampleError as exc:
        print(json.dumps({"type": "error", "error": str(exc)}), flush=True)
        return 1
    except (BrokenPipeError, KeyboardInterrupt):
        return 130
    return 0


if __name__ == "__main__":
    sys.exit(main())
