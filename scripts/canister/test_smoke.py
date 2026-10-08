"""Tests for accepting execution evidence, without network access."""

import copy
import json
import os
from pathlib import Path
import signal
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

import smoke


def local_context(args):
    if args == ("network", "status", "local", "--json"):
        return json.dumps({"api_url": "http://127.0.0.1:1234/"})
    if args == ("deploy", "-e", "local", "--json"):
        return json.dumps({"canisters": [{"name": "governance-probe", "canister_id": "aaaaa-aa"}]})
    raise AssertionError(f"unexpected discovery command: {args}")


def wait_for_file(path, process):
    deadline = time.monotonic() + 10
    while not path.exists() or path.stat().st_size == 0:
        if process.poll() is not None or time.monotonic() >= deadline:
            raise AssertionError("test subprocess did not reach the interruption point")
        time.sleep(0.01)


class ReceiptTests(unittest.TestCase):
    def setUp(self):
        self.payload = {
            "status": "ok",
            "report": {
                "schema_version": 1, "network": "ic",
                "governance_canister_id": smoke.GOVERNANCE,
                "fetched_at": "2026-09-15T12:00:00Z",
                "source": {
                    "source_transport": "replicated_inter_canister_call",
                    "collector_canister_id": "aaaaa-aa",
                },
                "economics": {"transaction_fee_e8s": 10_000},
            },
        }

    def test_probe_preserves_metadata_whitespace_through_the_helper_process(self):
        with tempfile.TemporaryDirectory() as directory:
            helper = Path(directory) / "helper"
            metadata = "service : { report : (text) -> (text); };\n"
            helper.write_text(f"#!{sys.executable}\nimport sys\nsys.stdout.write({metadata!r})\n")
            helper.chmod(0o700)
            with patch.object(smoke, "ARTIFACT_TOOL", helper):
                self.assertEqual(smoke.probe("metadata", "local", "http://127.0.0.1:1234/",
                                             "aaaaa-aa", "candid:service"), metadata)

    def test_icp_uses_repository_local_executable_and_project(self):
        with patch.dict(os.environ, {}, clear=True), \
                patch.object(smoke, "cargo_target_directory", return_value=smoke.ROOT / "target"), \
                patch.object(smoke, "run", return_value="running") as command:
            self.assertEqual(smoke.icp("network", "status", "local", timeout=30), "running")
        command.assert_called_once_with(
            str(smoke.ROOT / ".tools/ic/bin/icp"), "--project-root-override",
            str(smoke.PROJECT), "network", "status", "local", timeout=30,
            env={"ICP_HOME": str(smoke.ROOT / ".icp-smoke-home"), "DO_NOT_TRACK": "1"},
        )

    def test_icp_home_is_scoped_and_survives_disposable_state_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "consumer with spaces"
            project = root / "tests/canister"
            target = root / "configured Cargo output"
            project.mkdir(parents=True)
            tool = root / "icp-fixture"
            tool.write_text(f"#!{sys.executable}\n" + r'''
import json, os
from pathlib import Path
identity = Path(os.environ["ICP_HOME"]) / "identity/sentinel"
identity.parent.mkdir(parents=True, exist_ok=True)
identity.write_text("non-secret identity sentinel")
print(json.dumps({name: os.environ.get(name) for name in
                 ("ICP_HOME", "DO_NOT_TRACK", "XDG_DATA_HOME", "XDG_CONFIG_HOME",
                  "ICP_NETWORK", "ICP_ENVIRONMENT", "ICP_PROJECT_ROOT")}))
''')
            tool.chmod(0o700)
            caller = {"DO_NOT_TRACK": "0", "XDG_DATA_HOME": "caller data",
                      "XDG_CONFIG_HOME": "caller config", "ICP_NETWORK": "caller network",
                      "ICP_ENVIRONMENT": "caller environment", "ICP_PROJECT_ROOT": "caller project"}
            with patch.dict(os.environ, caller, clear=True), \
                    patch.object(smoke, "ROOT", root), patch.object(smoke, "PROJECT", project), \
                    patch.object(smoke, "ICP", tool), \
                    patch.object(smoke, "cargo_target_directory", return_value=target):
                result = json.loads(smoke.icp("network", "status", "local"))
                self.assertEqual(dict(os.environ), caller)
            self.assertEqual(result, {**caller, "DO_NOT_TRACK": "1",
                                      "ICP_HOME": str(root / ".icp-smoke-home"),
                                      "ICP_NETWORK": None, "ICP_ENVIRONMENT": None,
                                      "ICP_PROJECT_ROOT": None})
            for output in (root / "target", target, project / ".icp"):
                output.mkdir(parents=True)
                (output / "build-sentinel").write_text("disposable")
                shutil.rmtree(output)
            self.assertEqual((root / ".icp-smoke-home/identity/sentinel").read_text(),
                             "non-secret identity sentinel")

    def test_icp_preserves_explicit_persistent_home(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            selected = "persistent identity home"
            sentinel = root / selected / "identity/sentinel"
            sentinel.parent.mkdir(parents=True)
            sentinel.write_text("caller-owned sentinel")
            caller = {"ICP_HOME": selected, "DO_NOT_TRACK": "0"}
            with patch.dict(os.environ, caller, clear=True), \
                    patch.object(smoke, "ROOT", root), \
                    patch.object(smoke, "cargo_target_directory", return_value=root / "target"):
                self.assertEqual(smoke.icp_environment(), {**caller, "DO_NOT_TRACK": "1"})
                self.assertEqual(dict(os.environ), caller)
            self.assertEqual(sentinel.read_text(), "caller-owned sentinel")
            self.assertFalse((root / ".icp-smoke-home").exists())

    def test_icp_refuses_disposable_home_before_executable_dispatch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            project = root / "project"
            target = root / "configured Cargo output"
            (root / "output alias").symlink_to(root / "target", target_is_directory=True)
            selections = ("", "target", "target/missing identity home", "output alias/identity",
                          str(target / "identity"), str(project / ".icp/identity"))
            for selected in selections:
                with self.subTest(selected=selected), \
                        patch.dict(os.environ, {"ICP_HOME": selected}, clear=True), \
                        patch.object(smoke, "ROOT", root), patch.object(smoke, "PROJECT", project), \
                        patch.object(smoke, "cargo_target_directory", return_value=target), \
                        patch.object(smoke, "run") as command:
                    with self.assertRaisesRegex(ValueError, "persistent storage"):
                        smoke.icp("build", "-e", "local")
                    command.assert_not_called()
            self.assertFalse(target.exists())

    def test_build_wasm_preserves_caller_environment(self):
        caller = {"ICP_HOME": "caller home", "XDG_DATA_HOME": "caller data"}
        with patch.dict(os.environ, caller, clear=True), \
                patch.object(sys, "argv", ["smoke.py", "build-wasm"]), \
                patch.object(smoke, "build_wasm") as build:
            smoke.main()
            build.assert_called_once_with()
            self.assertEqual(dict(os.environ), caller)

    def test_build_checks_the_selected_toolset_before_icp(self):
        selected_pins = str(smoke.ROOT / "ci/ic-tools.tsv")
        for available in (True, False):
            with self.subTest(available=available), \
                    patch.dict(os.environ, {"IC_TOOL_PINS": selected_pins}), \
                    patch.object(sys, "argv", ["smoke.py", "build"]), \
                    patch.object(smoke, "run") as command, patch.object(smoke, "icp") as icp:
                if available:
                    command.return_value = ""
                    smoke.main()
                    icp.assert_called_once_with("build", "-e", "local", capture=False)
                else:
                    command.side_effect = subprocess.CalledProcessError(1, "tool check")
                    with self.assertRaises(subprocess.CalledProcessError):
                        smoke.main()
                    icp.assert_not_called()
                self.assertEqual(command.call_args_list[0].args, (
                    "bash", str(smoke.ROOT / "scripts/dev/install-ic-tools.sh"),
                    "--pins", selected_pins, "--check",
                ))

    def test_rejects_wrong_collector_transport_and_identity(self):
        for key, value in (("network", "unknown"), ("schema_version", 0),
                           ("governance_canister_id", "aaaaa-aa"),
                           ("source", {"source_transport": "replica_query"}),
                           ("source", {"source_transport": "replicated_inter_canister_call",
                                       "collector_canister_id": "2vxsx-fae"})):
            payload = copy.deepcopy(self.payload)
            payload["report"][key] = value
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                smoke.validate_report("economics", payload, "aaaaa-aa")

    def test_rejects_error_missing_payload_and_invalid_timestamp(self):
        with self.assertRaises(ValueError):
            smoke.validate_report("economics", {"status": "error", "error": "rejected"}, "aaaaa-aa")
        with self.assertRaises(ValueError):
            smoke.validate_report("metrics", self.payload, "aaaaa-aa")
        self.payload["report"]["fetched_at"] = "invalid"
        with self.assertRaises(ValueError):
            smoke.validate_report("economics", self.payload, "aaaaa-aa")

    def test_verifies_all_reports_and_detects_a_module_change(self):
        with tempfile.TemporaryDirectory() as directory:
            artifacts = Path(directory)
            (artifacts / "governance_probe.wasm").write_bytes(b"\0asm\x01\0\0\0")
            module_hash = smoke.sha256(b"\0asm\x01\0\0\0")
            replies = [module_hash, (smoke.PROJECT / "probe.did").read_text(), '{"schema_version": 1}']
            for kind in smoke.KINDS:
                payload = copy.deepcopy(self.payload)
                payload["report"][kind] = payload["report"].pop("economics")
                payload["report"][kind]["raw_amount"] = 2 ** 80
                replies.append(json.dumps(payload))
            for changed in (False, True):
                final = "changed" if changed else module_hash
                receipt = {"api_endpoint": "http://127.0.0.1:1234/"}
                with self.subTest(changed=changed), patch.object(smoke, "ARTIFACTS", artifacts), \
                        patch.object(smoke, "probe", side_effect=replies + [final]):
                    if changed:
                        with self.assertRaisesRegex(ValueError, "changed during collection"):
                            smoke.verify("local", "aaaaa-aa", receipt, artifacts / "receipt.json")
                    else:
                        smoke.verify("local", "aaaaa-aa", receipt, artifacts / "receipt.json")
                    self.assertEqual(set(receipt["reports"]), set(smoke.KINDS))
                    self.assertEqual(receipt["canister_id"], "aaaaa-aa")
                    self.assertEqual(receipt["module_hash_before"], module_hash)
                    self.assertEqual(receipt["module_hash_after"], final)
                    for kind in smoke.KINDS:
                        self.assertEqual(receipt["reports"][kind]["report"][kind]["raw_amount"], 2 ** 80)

    def test_mismatched_module_stops_before_any_report_call(self):
        with tempfile.TemporaryDirectory() as directory:
            artifacts = Path(directory)
            (artifacts / "governance_probe.wasm").write_bytes(b"\0asm\x01\0\0\0")
            with patch.object(smoke, "ARTIFACTS", artifacts), patch.object(
                smoke, "probe", return_value="wrong",
            ) as command:
                with self.assertRaisesRegex(ValueError, "module hash differs"):
                    smoke.verify("mainnet-smoke", "aaaaa-aa", {"api_endpoint": "https://icp-api.io/"}, artifacts / "receipt.json")
                self.assertEqual(command.call_count, 1)

    def test_invalid_local_wasm_stops_before_report_collection(self):
        with tempfile.TemporaryDirectory() as directory:
            artifacts = Path(directory)
            output = artifacts / "receipt.json"
            (artifacts / "governance_probe.wasm").write_bytes(b"invalid Wasm")
            with patch.object(smoke, "ARTIFACTS", artifacts), patch.object(
                smoke, "probe", return_value="anything",
            ) as command:
                with self.assertRaises(ValueError) as raised:
                    smoke.verify("local", "aaaaa-aa", {"api_endpoint": "http://127.0.0.1:1234/"}, output)
            self.assertIsInstance(raised.exception.__cause__, subprocess.CalledProcessError)
            command.assert_called_once()
            saved = json.loads(output.read_text())
            self.assertEqual(saved["phase"], "checking_module")
            self.assertEqual(saved["canister_id"], "aaaaa-aa")

    def test_failed_receipt_encoding_or_publication_preserves_the_previous_snapshot(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"
            receipt = {"schema_version": 1, "status": "running"}
            smoke.save_receipt(output, receipt, "starting_network")
            original = output.read_bytes()
            with patch.object(smoke, "admit_artifact") as publish:
                with self.assertRaises(ValueError):
                    smoke.save_receipt(output, dict(receipt, invalid=float("nan")), "deploying")
                publish.assert_not_called()
            with patch.object(smoke, "admit_artifact", side_effect=ValueError("disk failure")):
                with self.assertRaisesRegex(ValueError, "disk failure"):
                    smoke.save_receipt(output, receipt, "deploying")
            self.assertEqual(output.read_bytes(), original)
            self.assertEqual(list(Path(directory).iterdir()), [output])

    def test_receipt_replacement_remains_private_under_a_permissive_parent_umask(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"
            receipt = {"schema_version": 1, "status": "running"}
            smoke.save_receipt(output, receipt, "starting")
            previous_umask = os.umask(0)
            try:
                smoke.save_receipt(output, dict(receipt, status="passed"), "finished")
            finally:
                os.umask(previous_umask)
            self.assertEqual(output.stat().st_mode & 0o777, 0o600)
            self.assertEqual(json.loads(output.read_text())["status"], "passed")
            self.assertEqual(list(Path(directory).iterdir()), [output])

    def test_previous_report_and_invalid_reply_are_retained_before_validation(self):
        with tempfile.TemporaryDirectory() as directory:
            artifacts = Path(directory)
            output = artifacts / "receipt.json"
            (artifacts / "governance_probe.wasm").write_bytes(b"\0asm\x01\0\0\0")
            initial = iter([
                smoke.sha256(b"\0asm\x01\0\0\0"),
                (smoke.PROJECT / "probe.did").read_text(),
                '{"schema_version": 1}',
                json.dumps(self.payload),
            ])

            def command(*args, **_kwargs):
                if args[0] == "report" and args[4] == "metrics":
                    saved = json.loads(output.read_text())
                    self.assertEqual(saved["phase"], "collecting_metrics")
                    self.assertEqual(saved["reports"]["economics"]["report"], self.payload["report"])
                    Path(args[5]).write_bytes(b"invalid Candid")
                    raise ValueError("invalid Candid reply")
                return next(initial)

            with patch.object(smoke, "ARTIFACTS", artifacts), patch.object(smoke, "icp"), \
                    patch.object(smoke, "probe", side_effect=command):
                with self.assertRaises(ValueError) as raised:
                    smoke.run_attempt("mainnet-smoke", "aaaaa-aa", {"status": "running"}, output)
            saved = json.loads(output.read_text())
            self.assertEqual(saved["status"], "failed")
            self.assertEqual(saved["failed_phase"], "collecting_metrics")
            self.assertEqual(saved["error"], str(raised.exception))
            self.assertEqual(saved["reports"]["metrics"], {"reply_file": "metrics.candid"})
            self.assertEqual((artifacts / "metrics.candid").read_bytes(), b"invalid Candid")
            self.assertIn("report", saved["reports"]["economics"])


class LifecycleTests(unittest.TestCase):
    def test_successful_startup_hands_background_lifetime_to_network_cleanup(self):
        launcher = r'''
import os, sys, time
from pathlib import Path
directory = Path(sys.argv[1])
(directory / "launcher").write_text(str(os.getpid()))
while True:
    (directory / "heartbeat").write_text(str(time.monotonic_ns()))
    time.sleep(0.01)
'''
        start = r'''
import subprocess, sys, time
from pathlib import Path
subprocess.Popen([sys.executable, "-c", sys.argv[1], sys.argv[2]],
                 stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
while not (Path(sys.argv[2]) / "heartbeat").exists():
    time.sleep(0.01)
'''
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            receipt = path / "receipt.json"

            def command(*args, **kwargs):
                if args == ("network", "status", "local"):
                    raise subprocess.CalledProcessError(1, args)
                if args[:2] == ("network", "start"):
                    smoke.run(sys.executable, "-c", start, launcher, directory, **kwargs)
                elif args == ("network", "status", "local", "--json"):
                    heartbeat = (path / "heartbeat").read_bytes()
                    time.sleep(0.1)
                    self.assertNotEqual((path / "heartbeat").read_bytes(), heartbeat,
                                        "successful startup must not stop the owned background network")
                    return local_context(args)
                elif args[0] == "deploy":
                    return local_context(args)
                elif args[:2] == ("network", "stop"):
                    os.kill(int((path / "launcher").read_text()), signal.SIGTERM)

            try:
                with patch.object(smoke, "icp", side_effect=command), patch.object(smoke, "verify"):
                    smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, receipt)
                self.assertEqual(json.loads(receipt.read_text())["status"], "passed")
            finally:
                if (path / "launcher").exists():
                    try:
                        os.kill(int((path / "launcher").read_text()), signal.SIGKILL)
                    except ProcessLookupError:
                        pass

    def test_startup_failure_or_timeout_attempts_cleanup_and_retains_both_errors(self):
        for failure in (RuntimeError("startup failed"), subprocess.TimeoutExpired("icp", 900)):
            for cleanup_fails in (False, True):
                with self.subTest(failure=failure, cleanup_fails=cleanup_fails), tempfile.TemporaryDirectory() as directory:
                    output = Path(directory) / "receipt.json"
                    calls = []

                    def command(*args, **_kwargs):
                        calls.append(args)
                        if args[:2] == ("network", "status"):
                            raise subprocess.CalledProcessError(1, args)
                        saved = json.loads(output.read_text())
                        if args[:2] == ("network", "start"):
                            self.assertEqual(saved["phase"], "starting_network")
                            self.assertEqual(saved["status"], "running")
                            raise failure
                        self.assertEqual(args, ("network", "stop", "local"))
                        self.assertEqual(saved["status"], "failed")
                        if cleanup_fails:
                            raise RuntimeError("stop failed")

                    with patch.object(smoke, "icp", side_effect=command):
                        with self.assertRaises((RuntimeError, subprocess.TimeoutExpired)) as raised:
                            smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
                    self.assertIs(raised.exception, failure)
                    saved = json.loads(output.read_text())
                    self.assertEqual(saved["status"], "failed")
                    self.assertEqual(saved["failed_phase"], "starting_network")
                    self.assertEqual(saved["error"], str(failure))
                    self.assertEqual(saved["network_cleanup"], "failed" if cleanup_fails else "stopped")
                    self.assertEqual(calls[-1], ("network", "stop", "local"))
                    self.assertIn("finished_at", saved)
                    if cleanup_fails:
                        self.assertEqual(saved["cleanup_error"], "stop failed")

    def test_existing_network_is_neither_started_nor_stopped(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"
            with patch.object(smoke, "icp", return_value="running") as command:
                with self.assertRaisesRegex(ValueError, "already running"):
                    smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
                self.assertEqual(command.call_count, 1)
            self.assertNotIn("network_cleanup", json.loads(output.read_text()))

    def test_success_remains_running_until_cleanup_finishes(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"

            def command(*args, **_kwargs):
                if args == ("network", "status", "local"):
                    raise subprocess.CalledProcessError(1, args)
                if args[-1] == "--json":
                    return local_context(args)
                if args[:2] == ("network", "stop"):
                    self.assertEqual(json.loads(output.read_text())["status"], "running")

            with patch.object(smoke, "icp", side_effect=command), patch.object(smoke, "verify") as verify:
                smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
            self.assertEqual(verify.call_args.args[:2], ("local", "aaaaa-aa"))
            self.assertEqual(verify.call_args.args[2]["api_endpoint"], "http://127.0.0.1:1234/")
            saved = json.loads(output.read_text())
            self.assertEqual(saved["status"], "passed")
            self.assertEqual(saved["network_cleanup"], "stopped")

    def test_ambiguous_or_missing_deployment_identity_stops_verification_and_cleans_up(self):
        probe = {"name": "governance-probe", "canister_id": "aaaaa-aa"}
        for canisters in ([], [probe, probe], [{"name": "another-probe", "canister_id": "aaaaa-aa"}]):
            with self.subTest(canisters=canisters), tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / "receipt.json"

                def command(*args, **_kwargs):
                    if args == ("network", "status", "local"):
                        raise subprocess.CalledProcessError(1, args)
                    if args == ("deploy", "-e", "local", "--json"):
                        return json.dumps({"canisters": canisters})
                    if args[-1] == "--json":
                        return local_context(args)

                with patch.object(smoke, "icp", side_effect=command) as icp, \
                        patch.object(smoke, "verify") as verify:
                    with self.assertRaisesRegex(ValueError, "exactly one Governance probe"):
                        smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
                verify.assert_not_called()
                self.assertEqual(icp.call_args.args, ("network", "stop", "local"))
                saved = json.loads(output.read_text())
                self.assertEqual(saved["failed_phase"], "deploying")
                self.assertEqual(saved["status"], "failed")
                self.assertEqual(saved["network_cleanup"], "stopped")

    def test_startup_signals_stop_child_process_and_publish_failed_receipt(self):
        script = r'''
import json, signal, subprocess, sys, time
from pathlib import Path
import smoke
directory = Path(sys.argv[1])
def command(*args, **kwargs):
    if args[:2] == ("network", "status"):
        raise subprocess.CalledProcessError(1, args)
    if args[:2] == ("network", "start"):
        child_script = r"""
import os, signal, sys, time
from pathlib import Path
directory = Path(sys.argv[1])
if os.fork() == 0:
    while True:
        (directory / "descendant").write_text(str(time.monotonic_ns()))
        time.sleep(0.01)
(directory / "child").write_text(str(os.getpid()))
signal.pause()
"""
        smoke.run(sys.executable, "-c", child_script, str(directory))
    if args[:2] == ("network", "stop"):
        (directory / "stopped").write_text("yes")
        while not (directory / "release").exists():
            time.sleep(0.01)
smoke.icp = command
try:
    with smoke.handle_interrupts():
        smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, directory / "receipt.json")
except smoke.RunInterrupted as error:
    sys.exit(128 + error.signum)
'''
        for signum in (signal.SIGINT, signal.SIGTERM):
            with self.subTest(signal=signum), tempfile.TemporaryDirectory() as directory:
                path = Path(directory)
                with subprocess.Popen([sys.executable, "-c", script, directory], cwd=Path(smoke.__file__).parent,
                                      stdout=subprocess.DEVNULL, start_new_session=True) as process:
                    try:
                        wait_for_file(path / "child", process)
                        wait_for_file(path / "descendant", process)
                        child = int((path / "child").read_text())
                        os.killpg(process.pid, signum)
                        wait_for_file(path / "stopped", process)
                        descendant = (path / "descendant").read_bytes()
                        time.sleep(0.1)
                        self.assertEqual((path / "descendant").read_bytes(), descendant)
                        process.send_signal(signum)
                        (path / "release").touch()
                        self.assertEqual(process.wait(timeout=10), 128 + signum)
                        with self.assertRaises(ProcessLookupError):
                            os.kill(child, 0)
                        saved = json.loads((path / "receipt.json").read_text())
                        self.assertEqual(saved["status"], "failed")
                        self.assertEqual(saved["failed_phase"], "starting_network")
                        self.assertEqual(saved["interrupted_by"], signal.Signals(signum).name)
                        self.assertEqual(saved["network_cleanup"], "stopped")
                        self.assertTrue((path / "stopped").exists())
                    finally:
                        if process.poll() is None:
                            process.kill()
                        if (path / "child").exists():
                            try:
                                os.killpg(int((path / "child").read_text()), signal.SIGKILL)
                            except ProcessLookupError:
                                pass

    def test_cleanup_failure_cannot_mark_a_verified_run_passed(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"

            def command(*args, **_kwargs):
                if args == ("network", "status", "local"):
                    raise subprocess.CalledProcessError(1, args)
                if args[-1] == "--json":
                    return local_context(args)
                if args[:2] == ("network", "stop"):
                    raise RuntimeError("could not stop runtime")

            with patch.object(smoke, "icp", side_effect=command), patch.object(smoke, "verify"):
                with self.assertRaisesRegex(RuntimeError, "could not stop runtime"):
                    smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
            saved = json.loads(output.read_text())
            self.assertEqual(saved["status"], "failed")
            self.assertEqual(saved["failed_phase"], "stopping_network")
            self.assertEqual(saved["network_cleanup"], "failed")

    def test_receipt_io_failure_does_not_prevent_network_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"
            publish = smoke.save_receipt

            def checkpoint(path, receipt, phase=None):
                if phase == "stopping_network":
                    raise OSError("receipt storage unavailable")
                publish(path, receipt, phase)

            def command(*args, **_kwargs):
                if args == ("network", "status", "local"):
                    raise subprocess.CalledProcessError(1, args)
                if args[-1] == "--json":
                    return local_context(args)

            with patch.object(smoke, "icp", side_effect=command) as commands, \
                    patch.object(smoke, "verify"), patch.object(smoke, "save_receipt", side_effect=checkpoint):
                with self.assertRaisesRegex(OSError, "receipt storage unavailable"):
                    smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
            self.assertEqual(commands.call_args.args, ("network", "stop", "local"))
            saved = json.loads(output.read_text())
            self.assertEqual(saved["status"], "failed")
            self.assertEqual(saved["network_cleanup"], "stopped")

    def test_receipt_storage_failure_retains_the_original_operation_error(self):
        for recovered in (False, True):
            with self.subTest(storage_recovers=recovered), tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / "receipt.json"
                failure = RuntimeError("startup failed")
                publish = smoke.save_receipt

                def checkpoint(path, receipt, phase=None):
                    if receipt["status"] == "failed" and (not recovered or phase != "finished"):
                        raise OSError("receipt storage unavailable")
                    publish(path, receipt, phase)

                def command(*args, **_kwargs):
                    if args[:2] == ("network", "status"):
                        raise subprocess.CalledProcessError(1, args)
                    if args[:2] == ("network", "start"):
                        raise failure

                with patch.object(smoke, "icp", side_effect=command) as commands, \
                        patch.object(smoke, "save_receipt", side_effect=checkpoint):
                    with self.assertRaises(RuntimeError) as raised:
                        smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
                self.assertIs(raised.exception, failure)
                self.assertIn("receipt storage unavailable", failure.receipt_errors)
                self.assertEqual(commands.call_args.args, ("network", "stop", "local"))
                if recovered:
                    saved = json.loads(output.read_text())
                    self.assertEqual(saved["error"], "startup failed")
                    self.assertEqual(saved["network_cleanup"], "stopped")
                    self.assertIn("receipt storage unavailable", saved["receipt_errors"])
                else:
                    self.assertEqual(json.loads(output.read_text())["status"], "running")

    def test_final_receipt_failure_does_not_replace_a_standalone_network_cleanup_error(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"
            failure = RuntimeError("network cleanup failed")
            publish = smoke.save_receipt

            def checkpoint(path, receipt, phase=None):
                if phase == "finished":
                    raise OSError("final receipt unavailable")
                publish(path, receipt, phase)

            def command(*args, **_kwargs):
                if args == ("network", "status", "local"):
                    raise subprocess.CalledProcessError(1, args)
                if args[-1] == "--json":
                    return local_context(args)
                if args[:2] == ("network", "stop"):
                    raise failure

            with patch.object(smoke, "icp", side_effect=command), patch.object(smoke, "verify"), \
                    patch.object(smoke, "save_receipt", side_effect=checkpoint):
                with self.assertRaises(RuntimeError) as raised:
                    smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
            self.assertIs(raised.exception, failure)
            self.assertEqual(failure.receipt_errors, ["final receipt unavailable"])

    def test_command_timeout_reaps_its_child(self):
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "child"
            with self.assertRaises(subprocess.TimeoutExpired):
                smoke.run(sys.executable, "-c",
                          "import os,signal,sys; from pathlib import Path; "
                          "Path(sys.argv[1]).write_text(str(os.getpid())); signal.pause()",
                          str(marker), timeout=2)
            with self.assertRaises(ProcessLookupError):
                os.kill(int(marker.read_text()), 0)

    def test_escalation_stops_descendants_after_the_leader_exits(self):
        with tempfile.TemporaryDirectory() as directory:
            heartbeat = Path(directory) / "heartbeat"
            marker = Path(directory) / "descendant"
            child = r'''
import signal, sys, time
from pathlib import Path
signal.signal(signal.SIGTERM, signal.SIG_IGN)
while True:
    Path(sys.argv[1]).write_text(str(time.monotonic()))
    time.sleep(0.01)
'''
            leader = r'''
import signal, subprocess, sys, time
from pathlib import Path
child = subprocess.Popen([sys.executable, "-c", sys.argv[1], sys.argv[2]])
Path(sys.argv[3]).write_text(str(child.pid))
while not Path(sys.argv[2]).exists():
    time.sleep(0.01)
print("ready", flush=True)
signal.pause()
'''
            try:
                with self.assertRaises(subprocess.TimeoutExpired):
                    smoke.run(sys.executable, "-c", leader, child, str(heartbeat), str(marker), timeout=1)
                self.assertTrue(heartbeat.exists())
                stopped = heartbeat.read_bytes()
                time.sleep(0.1)
                self.assertEqual(heartbeat.read_bytes(), stopped)
            finally:
                if marker.exists():
                    try:
                        os.kill(int(marker.read_text()), signal.SIGKILL)
                    except ProcessLookupError:
                        pass

    def test_shared_cleanup_failures_are_retained_in_the_failed_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"
            failure = None
            errors = ["SIGTERM: refused", "SIGKILL: refused", "kill leader: refused", "reap leader: timed out"]
            outcome = {"failure": "timeout", "diagnostic": "deadline expired", "code": None,
                       "signal": None, "stdout": list(b"partial"), "stderr": None, "cleanup_errors": errors}

            def command(*args, **_kwargs):
                nonlocal failure
                if args[:2] == ("network", "status"):
                    raise subprocess.CalledProcessError(1, args)
                if args[:2] == ("network", "start"):
                    with patch.object(smoke, "process_outcome", return_value=outcome):
                        try:
                            smoke.run("probe", timeout=1)
                        except subprocess.TimeoutExpired as error:
                            failure = error
                            raise
                else:
                    self.assertEqual(args, ("network", "stop", "local"))
                    saved = json.loads(output.read_text())
                    self.assertEqual(saved["command_stdout"], "partial")
                    self.assertEqual(saved["command_cleanup_errors"], failure.command_cleanup_errors)

            with patch.object(smoke, "icp", side_effect=command):
                with self.assertRaises(subprocess.TimeoutExpired) as raised:
                    smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
            self.assertIs(raised.exception, failure)
            self.assertEqual(failure.command_cleanup_errors, errors)
            self.assertEqual(json.loads(output.read_text())["network_cleanup"], "stopped")

    def test_piped_input_and_both_outputs_are_drained_concurrently(self):
        script = r'''
import os, sys, threading
def output():
    sys.stdout.write("o" * 262144)
    sys.stdout.flush()
    sys.stdout.write(sys.stdin.read())
thread = threading.Thread(target=output)
thread.start()
sys.stderr.write("e" * 262144)
thread.join()
sys.exit(7)
'''
        contents = "piped receipt input\n" * 32768
        with self.assertRaises(subprocess.CalledProcessError) as raised:
            smoke.run(sys.executable, "-c", script, input=contents, stderr=subprocess.PIPE, timeout=10)
        self.assertEqual(raised.exception.returncode, 7)
        self.assertEqual(raised.exception.output, "o" * 262144 + contents)
        self.assertEqual(raised.exception.stderr, "e" * 262144)

    def test_inherited_outputs_are_not_captured_by_the_control_protocol(self):
        script = r'''
import smoke, sys
assert smoke.run(sys.executable, "-c", "import sys; print('streamed stdout'); print('streamed stderr', file=sys.stderr)",
                 capture=False) is None
'''
        result = subprocess.run([sys.executable, "-c", script], cwd=Path(smoke.__file__).parent,
                                text=True, capture_output=True, timeout=10, check=True)
        self.assertEqual(result.stdout, "streamed stdout\n")
        self.assertEqual(result.stderr, "streamed stderr\n")

    def test_timeout_retains_output_in_the_failed_receipt_before_network_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"
            failure = None

            def command(*args, **_kwargs):
                nonlocal failure
                if args[:2] == ("network", "status"):
                    raise subprocess.CalledProcessError(1, args)
                if args[:2] == ("network", "start"):
                    try:
                        smoke.run(sys.executable, "-c", "import signal,sys; print('startup output', flush=True); "
                                  "print('startup diagnostic', file=sys.stderr, flush=True); signal.pause()",
                                  timeout=1, stderr=subprocess.PIPE)
                    except subprocess.TimeoutExpired as error:
                        failure = error
                        raise
                self.assertEqual(args, ("network", "stop", "local"))
                saved = json.loads(output.read_text())
                self.assertEqual(saved["command_stdout"], "startup output\n")
                self.assertEqual(saved["command_stderr"], "startup diagnostic\n")
                raise RuntimeError("network cleanup failed")

            with patch.object(smoke, "icp", side_effect=command):
                with self.assertRaises(subprocess.TimeoutExpired) as raised:
                    smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
            self.assertIs(raised.exception, failure)
            self.assertEqual(json.loads(output.read_text())["cleanup_error"], "network cleanup failed")

    def test_timeout_does_not_wait_for_blocked_stdin(self):
        with self.assertRaises(subprocess.TimeoutExpired) as raised:
            smoke.run(sys.executable, "-c", "import signal; print('ready', flush=True); signal.pause()",
                      input="x" * 1048576, timeout=1)
        self.assertEqual(raised.exception.output, b"ready\n")

    def test_pipe_held_by_an_escaped_descendant_still_obeys_the_deadline(self):
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "escaped"
            leader = r'''
import subprocess, sys
from pathlib import Path
child = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(30)"], start_new_session=True)
Path(sys.argv[1]).write_text(str(child.pid))
print("leader exited", flush=True)
'''
            try:
                with self.assertRaises(subprocess.TimeoutExpired) as raised:
                    smoke.run(sys.executable, "-c", leader, str(marker), timeout=1)
                self.assertEqual(raised.exception.output, b"leader exited\n")
            finally:
                if marker.exists():
                    try:
                        os.kill(int(marker.read_text()), signal.SIGKILL)
                    except ProcessLookupError:
                        pass

    def test_invalid_utf8_cannot_replace_an_existing_timeout(self):
        with self.assertRaises(subprocess.TimeoutExpired) as raised:
            smoke.run(sys.executable, "-c", "import os,signal; os.write(1, b'prefix\\xff'); signal.pause()",
                      timeout=1)
        self.assertTrue(raised.exception.output.startswith(b"prefix"))

    def test_successful_command_admits_text_and_normalizes_newlines(self):
        self.assertEqual(smoke.run(sys.executable, "-c", "import os; os.write(1, b'first\\r\\nsecond\\r')",
                                   strip=False), "first\nsecond\n")
        with self.assertRaises(UnicodeDecodeError):
            smoke.run(sys.executable, "-c", "import os; os.write(2, b'\\xff')", stderr=subprocess.PIPE)

    def test_failed_text_admission_cleans_up_before_background_handoff(self):
        launcher = r'''
import os, sys, time
from pathlib import Path
path = Path(sys.argv[1])
(path / "child").write_text(str(os.getpid()))
while True:
    (path / "heartbeat").write_text(str(time.monotonic_ns()))
    time.sleep(0.01)
'''
        start = r'''
import os, subprocess, sys, time
from pathlib import Path
subprocess.Popen([sys.executable, "-c", sys.argv[1], sys.argv[2]],
                 stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
while not (Path(sys.argv[2]) / "heartbeat").exists():
    time.sleep(0.01)
os.write(int(sys.argv[3]), b'prefix\xff')
'''
        for descriptor in (1, 2):
            with self.subTest(descriptor=descriptor), tempfile.TemporaryDirectory() as directory:
                path = Path(directory)
                try:
                    with self.assertRaisesRegex(RuntimeError, "invalid command text") as raised:
                        smoke.run(sys.executable, "-c", start, launcher, directory, str(descriptor),
                                  stderr=subprocess.PIPE, handoff=True, timeout=10)
                    evidence = raised.exception.output if descriptor == 1 else raised.exception.stderr
                    self.assertEqual(evidence, "prefix\ufffd")
                    heartbeat = (path / "heartbeat").read_bytes()
                    time.sleep(0.1)
                    self.assertEqual((path / "heartbeat").read_bytes(), heartbeat)
                finally:
                    if (path / "child").exists():
                        try:
                            os.kill(int((path / "child").read_text()), signal.SIGKILL)
                        except ProcessLookupError:
                            pass

    def test_hard_termination_preserves_the_last_complete_snapshot(self):
        script = r'''
import signal, sys
from pathlib import Path
import smoke
output = Path(sys.argv[1]) / "receipt.json"
smoke.save_receipt(output, {"schema_version": 1, "status": "running", "reports": {
    "economics": {"report": {"transaction_fee_e8s": 10000}}
}}, "collecting_metrics")
signal.pause()
'''
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"
            with subprocess.Popen([sys.executable, "-c", script, directory], cwd=Path(smoke.__file__).parent) as process:
                try:
                    wait_for_file(output, process)
                    before = output.read_bytes()
                    process.kill()
                    process.wait(timeout=10)
                    self.assertEqual(output.read_bytes(), before)
                    saved = json.loads(before)
                    self.assertEqual(saved["status"], "running")
                    self.assertEqual(saved["phase"], "collecting_metrics")
                    self.assertIn("economics", saved["reports"])
                finally:
                    if process.poll() is None:
                        process.kill()


if __name__ == "__main__":
    unittest.main()
