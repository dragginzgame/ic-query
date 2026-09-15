#!/usr/bin/env python3
"""Build, bundle, and verify the isolated Governance canister probe."""

import argparse
from contextlib import contextmanager
import datetime
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
PROJECT = ROOT / "tests/canister"
ARTIFACTS = ROOT / "target/canister-smoke"
GOVERNANCE = "rrkah-fqaaa-aaaaa-aaaaq-cai"
KINDS = ("economics", "metrics", "reward_event", "maturity_modulation")


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


def run(*args, capture=True, timeout=600, stderr=None):
    with subprocess.Popen(
        args, cwd=ROOT, text=True, start_new_session=True,
        stdout=subprocess.PIPE if capture else None, stderr=stderr,
    ) as process:
        try:
            stdout, _ = process.communicate(timeout=timeout)
            if process.returncode:
                raise subprocess.CalledProcessError(process.returncode, args, output=stdout)
        except BaseException:
            # Stop only this command's process group before network-level cleanup.
            try:
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=5)
            except (ProcessLookupError, subprocess.TimeoutExpired):
                pass
            finally:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                process.wait()
            raise
    return stdout.strip() if capture else None


def icp(*args, **kwargs):
    return run("icp", "--project-root-override", str(PROJECT), *args, **kwargs)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def save_receipt(output, receipt, phase=None):
    """Publish one complete snapshot without truncating the previous receipt."""
    if phase is not None:
        receipt["phase"] = phase
    receipt["updated_at"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=output.parent, prefix=".receipt-", delete=False,
        ) as stream:
            temporary = Path(stream.name)
            json.dump(receipt, stream, indent=2, allow_nan=False)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, output)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


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


def decode_text_reply(envelope):
    """Decode exactly the probe's one-text Candid response, rejecting extra data."""
    raw = bytes.fromhex(envelope["response_bytes"])
    if not raw.startswith(b"DIDL\x00\x01\x71"):
        raise ValueError("probe must return exactly one Candid text value")
    offset, size, shift = 7, 0, 0
    while True:
        if offset >= len(raw) or shift > 63:
            raise ValueError("invalid Candid text length")
        byte = raw[offset]
        offset += 1
        size |= (byte & 127) << shift
        if byte < 128:
            break
        shift += 7
    if len(raw) - offset != size:
        raise ValueError("Candid text length does not match response")
    return json.loads(raw[offset:].decode())


def build_wasm():
    run(
        "cargo", "build", "-p", "ic-query", "--example", "governance_probe",
        "--target", "wasm32-unknown-unknown", "--release", "--no-default-features",
        "--features", "canister", "--locked", capture=False,
    )
    target = Path(json.loads(run(
        "cargo", "metadata", "--format-version", "1", "--no-deps", "--locked",
    ))["target_directory"])
    wasm = (target / "wasm32-unknown-unknown/release/examples/governance_probe.wasm").read_bytes()
    if wasm[:8] != b"\x00asm\x01\x00\x00\x00":
        raise ValueError("build output is not a core Wasm module")
    paths = sorted(set(
        [ROOT / name for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml")]
        + list((ROOT / "crates/ic-query").rglob("*.rs"))
        + [ROOT / "crates/ic-query/Cargo.toml", PROJECT / "probe.did", Path(__file__).resolve()]
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
    selection = ("-e", environment)
    status_args = ("canister", "status", canister, "--public", "--json", *selection)
    save_receipt(output, receipt, "checking_module")
    before = json.loads(icp(*status_args))
    receipt["canister_before"] = before
    save_receipt(output, receipt)
    expected_hash = "0x" + sha256((ARTIFACTS / "governance_probe.wasm").read_bytes())
    if before["module_hash"] != expected_hash:
        raise ValueError("deployed module hash differs from the locally built probe")
    save_receipt(output, receipt, "checking_candid")
    metadata = json.loads(icp(
        "canister", "metadata", canister, "candid:service", "--json", *selection,
    ))["value"]
    if metadata != (PROJECT / "probe.did").read_text():
        raise ValueError("deployed Candid metadata differs from probe.did")
    receipt["candid_sha256"] = sha256(metadata.encode())
    save_receipt(output, receipt, "checking_build_metadata")
    receipt["build"] = json.loads(json.loads(icp(
        "canister", "metadata", canister, "ic-query:build", "--json", *selection,
    ))["value"])
    receipt["reports"] = {}
    save_receipt(output, receipt)
    for kind in KINDS:
        print(f"Collecting {kind} ({environment})", flush=True)
        save_receipt(output, receipt, f"collecting_{kind}")
        envelope = json.loads(icp(
            "canister", "call", canister, "report", f'("{kind}")',
            "--candid", str(PROJECT / "probe.did"), "--json", *selection,
        ))
        entry = {"response": envelope}
        receipt["reports"][kind] = entry
        save_receipt(output, receipt, f"validating_{kind}")
        entry["report"] = validate_report(kind, decode_text_reply(envelope), before["id"])
        save_receipt(output, receipt, f"collected_{kind}")
    save_receipt(output, receipt, "checking_final_module")
    after = json.loads(icp(*status_args))
    receipt["canister_after"] = after
    save_receipt(output, receipt)
    if before["id"] != after["id"] or before["module_hash"] != after["module_hash"]:
        raise ValueError("probe identity or module changed during collection")


def run_attempt(environment, canister, receipt, output):
    startup_attempted = False
    verified = False
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
            save_receipt(output, receipt, "deploying")
            icp("deploy", "-e", "local", capture=False)
        else:
            receipt["api_endpoint"] = "https://icp-api.io/"
            save_receipt(output, receipt, "building")
            icp("build", "-e", environment, capture=False)
        verify(environment, canister, receipt, output)
        verified = True
    except BaseException as error:
        receipt["status"] = "failed"
        receipt["error"] = str(error) or type(error).__name__
        receipt["failed_phase"] = receipt["phase"]
        if isinstance(error, RunInterrupted):
            receipt["interrupted_by"] = signal.Signals(error.signum).name
        save_receipt(output, receipt)
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
            raise
        finally:
            if verified and receipt["status"] == "running":
                receipt["status"] = "passed"
            receipt["finished_at"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
            save_receipt(output, receipt, "finished")


def main():
    # Keep tool settings, identities, and launcher downloads inside the checkout.
    for name, directory in (("XDG_DATA_HOME", "data"), ("XDG_CACHE_HOME", "cache"),
                            ("XDG_CONFIG_HOME", "config")):
        os.environ[name] = str(ARTIFACTS / directory)
    os.environ["DO_NOT_TRACK"] = "1"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("build-wasm", "build", "bundle", "local", "verify-mainnet"))
    parser.add_argument("--canister", help="existing mainnet probe principal; no deployment is performed")
    args = parser.parse_args()
    if args.action == "build-wasm":
        build_wasm()
        return
    if run("icp", "--version") != "icp 1.5.0":
        parser.error("this harness requires ICP CLI 1.5.0")
    if args.action == "verify-mainnet" and not args.canister:
        parser.error("verify-mainnet requires --canister")
    if args.canister and args.action != "verify-mainnet":
        parser.error("--canister is only valid for verify-mainnet")
    # Explicit selections below must not inherit a caller's network override.
    for name in ("ICP_NETWORK", "ICP_ENVIRONMENT", "ICP_PROJECT_ROOT"):
        os.environ.pop(name, None)
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
        "icp": run("icp", "--version"), "rustc": run("rustc", "--version"),
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
