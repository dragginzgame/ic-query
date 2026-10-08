#!/usr/bin/env python3
"""Build, bundle, and verify the isolated Governance canister probe."""

import argparse
from contextlib import contextmanager
import datetime
from functools import cache
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
PROJECT = ROOT / "tests/canister"
ARTIFACTS = ROOT / "target/canister-smoke"
GOVERNANCE = "rrkah-fqaaa-aaaaa-aaaaq-cai"
KINDS = ("economics", "metrics", "reward_event", "maturity_modulation")
ARTIFACT_TOOL = None
ICP = ROOT / ".tools/ic/bin/icp"


class RunInterrupted(Exception):
    """A termination request that still allows receipt publication and cleanup."""

    def __init__(self, signum):
        self.signum = signum
        super().__init__(f"interrupted by {signal.Signals(signum).name}")


@contextmanager
def handle_interrupts():
    def interrupted(signum, _frame):
        # Further signals must not interrupt the first signal's cleanup.
        for item in (signal.SIGINT, signal.SIGTERM):
            signal.signal(item, signal.SIG_IGN)
        raise RunInterrupted(signum)

    previous = {item: signal.signal(item, interrupted)
                for item in (signal.SIGINT, signal.SIGTERM)}
    try:
        yield
    finally:
        for item, handler in previous.items():
            signal.signal(item, handler)


def run(*args, capture=True, timeout=600, stderr=None, input=None, umask=-1, env=None, strip=True):
    process = subprocess.Popen(
        args, cwd=ROOT, text=True, start_new_session=True,
        stdin=subprocess.PIPE if input is not None else None,
        stdout=subprocess.PIPE if capture else None, stderr=stderr,
        umask=umask, env=env,
    )
    operation_error = None

    def cleanup_failure(stage, error):
        message = f"{stage}: {str(error) or type(error).__name__}"
        if not hasattr(operation_error, "command_cleanup_errors"):
            operation_error.command_cleanup_errors = []
        operation_error.command_cleanup_errors.append(message)

    try:
        stdout, diagnostic = process.communicate(input=input, timeout=timeout)
        if process.returncode:
            raise subprocess.CalledProcessError(
                process.returncode, args, output=stdout, stderr=diagnostic,
            )
    except BaseException as error:
        operation_error = error
        # A reaped leader no longer reserves its PID for safe group signalling.
        if process.returncode is None:
            for signum in (signal.SIGTERM, signal.SIGKILL):
                try:
                    os.killpg(process.pid, signum)
                    if signum == signal.SIGTERM:
                        # Retain the leader's PID through escalation, even if it exits.
                        time.sleep(5)
                except ProcessLookupError:
                    pass
                except BaseException as cleanup_error:
                    cleanup_failure(signal.Signals(signum).name, cleanup_error)
                    if signum == signal.SIGKILL:
                        try:
                            process.kill()
                        except BaseException as kill_error:
                            cleanup_failure("kill leader", kill_error)
            try:
                process.wait(timeout=5)
            except BaseException as cleanup_error:
                cleanup_failure("reap leader", cleanup_error)
        raise
    finally:
        # Do not drain inherited pipes or perform an unbounded context-manager wait.
        for pipe in (process.stdin, process.stdout, process.stderr):
            if pipe is not None:
                try:
                    pipe.close()
                except BaseException as cleanup_error:
                    if operation_error is None:
                        raise
                    cleanup_failure("close command pipe", cleanup_error)
    return (stdout.strip() if strip else stdout) if capture else None


def icp(*args, **kwargs):
    return run(str(ICP), "--project-root-override", str(PROJECT), *args,
               env=icp_environment(), **kwargs)


def icp_environment():
    """Keep ICP global state outside build/runtime resets, scoped to its child."""
    environment = os.environ.copy()
    selected = environment.get("ICP_HOME", str(ROOT / ".icp-smoke-home"))
    if not selected:
        raise ValueError("ICP_HOME must select persistent storage outside build/runtime state")
    identity_home = (ROOT / selected).resolve()
    disposable = (ROOT / "target", cargo_target_directory(), PROJECT / ".icp")
    if any(identity_home.is_relative_to(path.resolve()) for path in disposable):
        raise ValueError("ICP_HOME must select persistent storage outside build/runtime state")
    environment["ICP_HOME"] = selected
    environment["DO_NOT_TRACK"] = "1"
    # Explicit project/network arguments own these selections in the child.
    for name in ("ICP_NETWORK", "ICP_ENVIRONMENT", "ICP_PROJECT_ROOT"):
        environment.pop(name, None)
    return environment


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def save_receipt(output, receipt, phase=None):
    """Publish one complete snapshot without truncating the previous receipt."""
    if phase is not None:
        receipt["phase"] = phase
    receipt["updated_at"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    contents = json.dumps(receipt, indent=2, allow_nan=False) + "\n"
    admit_artifact("write-receipt", str(output), input=contents)


def leb128(number):
    output = bytearray()
    while number >= 128:
        output.append((number & 127) | 128)
        number >>= 7
    output.append(number)
    return bytes(output)


def metadata_section(name, contents):
    name = ("icp:public " + name).encode()
    payload = leb128(len(name)) + name + contents
    return b"\x00" + leb128(len(payload)) + payload


@cache
def cargo_target_directory():
    return Path(json.loads(run(
        "cargo", "metadata", "--format-version", "1", "--no-deps", "--locked", "--offline",
    ))["target_directory"])


def admit_artifact(*args, input=None, strip=True):
    """Use the development-only shared host boundary; keep orchestration here."""
    global ARTIFACT_TOOL
    if ARTIFACT_TOOL is None:
        run("cargo", "build", "-p", "ic-query-cli", "--example", "governance_artifact",
            "--locked", "--offline", capture=False)
        ARTIFACT_TOOL = cargo_target_directory() / "debug/examples/governance_artifact"
    try:
        return run(str(ARTIFACT_TOOL), *args, input=input, stderr=subprocess.PIPE, umask=0o077, strip=strip)
    except subprocess.CalledProcessError as error:
        diagnostic = (error.stderr or "").strip()
        message = "Governance smoke helper failed"
        if diagnostic:
            message += f": {diagnostic}"
        raise ValueError(message) from error


def probe(command, environment, endpoint, canister, *args):
    return admit_artifact(command, environment, endpoint, canister, *args, strip=False)


def build_wasm():
    run(
        "cargo", "build", "-p", "ic-query", "--example", "governance_probe",
        "--target", "wasm32-unknown-unknown", "--release", "--no-default-features",
        "--features", "canister", "--locked", "--offline", capture=False,
    )
    wasm_path = cargo_target_directory() / "wasm32-unknown-unknown/release/examples/governance_probe.wasm"
    admitted_hash = admit_artifact("inspect-wasm", str(wasm_path))
    wasm = wasm_path.read_bytes()
    if sha256(wasm) != admitted_hash:
        raise ValueError("build output changed after Wasm inspection")
    paths = sorted(set(
        [ROOT / name for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml")]
        + list((ROOT / "crates/ic-query").rglob("*.rs"))
        + [ROOT / "crates/ic-query/Cargo.toml", ROOT / "crates/ic-query-cli/Cargo.toml",
           ROOT / "crates/ic-query-cli/examples/governance_artifact.rs",
           PROJECT / "probe.did", Path(__file__).resolve()]
    ))
    sources = hashlib.sha256()
    for path in paths:
        sources.update(str(path.relative_to(ROOT)).encode() + b"\x00")
        sources.update(path.read_bytes() + b"\x00")
    build = {
        "schema_version": 1,
        "rustc": run("rustc", "--version"),
        "cargo_lock_sha256": sha256((ROOT / "Cargo.lock").read_bytes()),
        "source_sha256": sources.hexdigest(),
    }
    wasm += metadata_section("candid:service", (PROJECT / "probe.did").read_bytes())
    wasm += metadata_section("ic-query:build", json.dumps(build, sort_keys=True).encode())
    ARTIFACTS.mkdir(parents=True, exist_ok=True)
    (ARTIFACTS / "governance_probe.wasm").write_bytes(wasm)
    Path(os.environ["ICP_WASM_OUTPUT_PATH"]).write_bytes(wasm)


def validate_report(kind, payload, collector):
    if payload.get("status") != "ok":
        raise ValueError(f"{kind}: {payload.get('error', 'missing successful response')}")
    report = payload["report"]
    expected = {
        "source_transport": "replicated_inter_canister_call",
        "collector_canister_id": collector,
    }
    if (report.get("schema_version") != 1 or report.get("network") != "ic"
            or report.get("governance_canister_id") != GOVERNANCE
            or report.get("source") != expected or kind not in report):
        raise ValueError(f"{kind}: incorrect report identity or replicated provenance")
    datetime.datetime.strptime(report["fetched_at"], "%Y-%m-%dT%H:%M:%SZ")
    return report


def verify(environment, canister, receipt, output):
    endpoint = receipt["api_endpoint"]
    receipt["canister_id"] = canister
    save_receipt(output, receipt, "checking_module")
    before = probe("module-hash", environment, endpoint, canister)
    receipt["module_hash_before"] = before
    save_receipt(output, receipt)
    expected_hash = admit_artifact(
        "inspect-wasm", str(ARTIFACTS / "governance_probe.wasm"),
    )
    if before != expected_hash:
        raise ValueError("deployed module hash differs from the locally built probe")
    save_receipt(output, receipt, "checking_candid")
    metadata = probe("metadata", environment, endpoint, canister, "candid:service")
    if metadata != (PROJECT / "probe.did").read_text():
        raise ValueError("deployed Candid metadata differs from probe.did")
    receipt["candid_sha256"] = sha256(metadata.encode())
    save_receipt(output, receipt, "checking_build_metadata")
    receipt["build"] = json.loads(probe("metadata", environment, endpoint, canister, "ic-query:build"))
    receipt["reports"] = {}
    save_receipt(output, receipt)
    for kind in KINDS:
        print(f"Collecting {kind} ({environment})", flush=True)
        save_receipt(output, receipt, f"collecting_{kind}")
        reply = f"{kind}.candid"
        entry = {"reply_file": reply}
        receipt["reports"][kind] = entry
        save_receipt(output, receipt)
        response = probe("report", environment, endpoint, canister, kind, str(output.parent / reply))
        save_receipt(output, receipt, f"validating_{kind}")
        entry["report"] = validate_report(kind, json.loads(response), canister)
        save_receipt(output, receipt, f"collected_{kind}")
    save_receipt(output, receipt, "checking_final_module")
    after = probe("module-hash", environment, endpoint, canister)
    receipt["module_hash_after"] = after
    save_receipt(output, receipt)
    if before != after:
        raise ValueError("probe module changed during collection")


def run_attempt(environment, canister, receipt, output):
    startup_attempted = False
    verified = False
    operation_error = None
    save_receipt(output, receipt, "starting")
    print(f"Receipt: {output}", flush=True)
    try:
        if environment == "local":
            save_receipt(output, receipt, "checking_network")
            try:
                icp("network", "status", "local", timeout=30, stderr=subprocess.DEVNULL)
            except subprocess.CalledProcessError:
                pass
            else:
                raise ValueError("the harness network is already running; stop it before running the smoke")
            save_receipt(output, receipt, "starting_network")
            # Startup can create a background launcher before returning to us.
            startup_attempted = True
            icp("network", "start", "local", "--background", capture=False, timeout=900)
            save_receipt(output, receipt, "reading_network")
            receipt["network"] = json.loads(icp("network", "status", "local", "--json"))
            receipt["api_endpoint"] = receipt["network"]["api_url"]
            save_receipt(output, receipt, "deploying")
            deployment = json.loads(icp("deploy", "-e", "local", "--json"))
            deployed = [item["canister_id"] for item in deployment["canisters"] if item["name"] == "governance-probe"]
            if len(deployed) != 1:
                raise ValueError("deployment must identify exactly one Governance probe")
            canister = deployed[0]
        else:
            receipt["api_endpoint"] = "https://icp-api.io/"
            save_receipt(output, receipt, "building")
            icp("build", "-e", environment, capture=False)
        verify(environment, canister, receipt, output)
        verified = True
    except BaseException as error:
        operation_error = error
        receipt["status"] = "failed"
        receipt["error"] = str(error) or type(error).__name__
        receipt["failed_phase"] = receipt["phase"]
        if isinstance(error, RunInterrupted):
            receipt["interrupted_by"] = signal.Signals(error.signum).name
        if getattr(error, "command_cleanup_errors", None):
            receipt["command_cleanup_errors"] = error.command_cleanup_errors
        for name in ("output", "stderr"):
            value = getattr(error, name, None)
            if isinstance(value, bytes):
                value = value.decode("utf-8", errors="replace")
            if value is not None:
                receipt["command_stdout" if name == "output" else "command_stderr"] = value
        try:
            save_receipt(output, receipt)
        except BaseException as storage_error:
            receipt.setdefault("receipt_errors", []).append(str(storage_error) or type(storage_error).__name__)
            error.receipt_errors = receipt["receipt_errors"]
        raise
    finally:
        try:
            if startup_attempted:
                receipt["network_cleanup"] = "pending"
                try:
                    save_receipt(output, receipt, "stopping_network")
                finally:
                    # Receipt IO failures must not prevent runtime cleanup.
                    try:
                        icp("network", "stop", "local", capture=False, timeout=30)
                    except BaseException:
                        receipt["network_cleanup"] = "failed"
                        raise
                    else:
                        receipt["network_cleanup"] = "stopped"
        except BaseException as error:
            receipt["status"] = "failed"
            receipt.setdefault("failed_phase", receipt["phase"])
            if isinstance(error, RunInterrupted):
                receipt["interrupted_by"] = signal.Signals(error.signum).name
            receipt["cleanup_error"] = str(error) or type(error).__name__
            if operation_error is None:
                operation_error = error
                raise
        finally:
            if verified and receipt["status"] == "running":
                receipt["status"] = "passed"
            receipt["finished_at"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
            try:
                save_receipt(output, receipt, "finished")
            except BaseException as storage_error:
                if operation_error is None:
                    raise
                receipt.setdefault("receipt_errors", []).append(str(storage_error) or type(storage_error).__name__)
                operation_error.receipt_errors = receipt["receipt_errors"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("build-wasm", "build", "bundle", "local", "verify-mainnet"))
    parser.add_argument("--canister", help="existing mainnet probe principal; no deployment is performed")
    args = parser.parse_args()
    if args.action == "build-wasm":
        build_wasm()
        return
    run("bash", str(ROOT / "scripts/dev/install-ic-tools.sh"),
        "--pins", os.environ.get("IC_TOOL_PINS", str(ROOT / "ci/ic-tools.tsv")), "--check")
    if args.action == "verify-mainnet" and not args.canister:
        parser.error("verify-mainnet requires --canister")
    if args.canister and args.action != "verify-mainnet":
        parser.error("--canister is only valid for verify-mainnet")
    if args.action in ("build", "bundle"):
        if args.action == "build":
            icp("build", "-e", "local", capture=False)
        else:
            ARTIFACTS.mkdir(parents=True, exist_ok=True)
            icp("project", "bundle", "-e", "mainnet-smoke", "--output",
                str(ARTIFACTS / "governance-probe.tar.gz"), capture=False)
        return
    ARTIFACTS.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="receipt-", dir=ARTIFACTS)) / "receipt.json"
    environment = "local" if args.action == "local" else "mainnet-smoke"
    receipt = {
        "schema_version": 1, "status": "running", "environment": environment,
        "evidence_scope": "local_nns" if environment == "local" else "mainnet",
        "icp": run(str(ICP), "--version"), "rustc": run("rustc", "--version"),
        "started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "point_in_time_guaranteed": False,
    }
    run_attempt(environment, args.canister or "governance-probe", receipt, output)


if __name__ == "__main__":
    try:
        with handle_interrupts():
            main()
    except RunInterrupted as error:
        print(error, file=sys.stderr)
        sys.exit(128 + error.signum)
