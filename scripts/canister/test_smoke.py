"""Tests for accepting execution evidence, without network access."""

import copy
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

import smoke


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

    def test_icp_uses_repository_local_executable_and_project(self):
        with patch.object(smoke, "run", return_value="running") as command:
            self.assertEqual(smoke.icp("network", "status", "local", timeout=30), "running")
        command.assert_called_once_with(
            str(smoke.ROOT / ".tools/ic/bin/icp"), "--project-root-override",
            str(smoke.PROJECT), "network", "status", "local", timeout=30,
        )

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

    def test_accepts_exact_text_reply_and_preserves_raw_report(self):
        text = json.dumps(self.payload).encode()
        raw = b"DIDL\x00\x01\x71" + smoke.leb128(len(text)) + text
        decoded = smoke.decode_text_reply(json.dumps({"response_bytes": raw.hex()}))
        report = smoke.validate_report("economics", decoded, "aaaaa-aa")
        self.assertEqual(report["economics"]["transaction_fee_e8s"], 10_000)

    def test_rejects_truncated_extra_and_wrong_type_replies(self):
        for raw in (b"DIDL\x00\x01\x71\x05{}", b"DIDL\x00\x01\x71\x02{}x",
                    b"DIDL\x00\x01\x71\x80", b"DIDL\x00\x01\x7e\x00"):
            with self.subTest(raw=raw), self.assertRaises(ValueError) as raised:
                smoke.decode_text_reply(json.dumps({"response_bytes": raw.hex()}))
            cause = raised.exception.__cause__
            self.assertIsInstance(cause, subprocess.CalledProcessError)
            self.assertTrue(cause.stderr)
            self.assertIn(cause.stderr.strip(), str(raised.exception))

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
            status = {"id": "aaaaa-aa", "module_hash": "0x" + smoke.sha256(b"\0asm\x01\0\0\0")}
            replies = [json.dumps(status),
                       json.dumps({"value": (smoke.PROJECT / "probe.did").read_text()}),
                       json.dumps({"value": '{"schema_version": 1}'})]
            for kind in smoke.KINDS:
                payload = copy.deepcopy(self.payload)
                payload["report"][kind] = payload["report"].pop("economics")
                text = json.dumps(payload).encode()
                raw = b"DIDL\x00\x01\x71" + smoke.leb128(len(text)) + text
                replies.append(json.dumps({"response_bytes": raw.hex()}))
            for changed in (False, True):
                final = dict(status, module_hash="0xchanged") if changed else status
                receipt = {}
                with self.subTest(changed=changed), patch.object(smoke, "ARTIFACTS", artifacts), \
                        patch.object(smoke, "icp", side_effect=replies + [json.dumps(final)]):
                    if changed:
                        with self.assertRaisesRegex(ValueError, "changed during collection"):
                            smoke.verify("local", "governance-probe", receipt, artifacts / "receipt.json")
                    else:
                        smoke.verify("local", "governance-probe", receipt, artifacts / "receipt.json")
                    self.assertEqual(set(receipt["reports"]), set(smoke.KINDS))

    def test_mismatched_module_stops_before_any_report_call(self):
        with tempfile.TemporaryDirectory() as directory:
            artifacts = Path(directory)
            (artifacts / "governance_probe.wasm").write_bytes(b"\0asm\x01\0\0\0")
            with patch.object(smoke, "ARTIFACTS", artifacts), patch.object(
                smoke, "icp", return_value='{"id": "aaaaa-aa", "module_hash": "0xwrong"}',
            ) as command:
                with self.assertRaisesRegex(ValueError, "module hash differs"):
                    smoke.verify("mainnet-smoke", "aaaaa-aa", {}, artifacts / "receipt.json")
                self.assertEqual(command.call_count, 1)

    def test_invalid_local_wasm_stops_before_report_collection(self):
        with tempfile.TemporaryDirectory() as directory:
            artifacts = Path(directory)
            output = artifacts / "receipt.json"
            (artifacts / "governance_probe.wasm").write_bytes(b"invalid Wasm")
            with patch.object(smoke, "ARTIFACTS", artifacts), patch.object(
                smoke, "icp", return_value='{"id": "aaaaa-aa", "module_hash": "0xanything"}',
            ) as command:
                with self.assertRaises(ValueError) as raised:
                    smoke.verify("local", "governance-probe", {}, output)
            self.assertIsInstance(raised.exception.__cause__, subprocess.CalledProcessError)
            command.assert_called_once()
            saved = json.loads(output.read_text())
            self.assertEqual(saved["phase"], "checking_module")
            self.assertEqual(saved["canister_before"]["id"], "aaaaa-aa")

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
            text = json.dumps(self.payload).encode()
            reply = {"response_bytes": (b"DIDL\x00\x01\x71" + smoke.leb128(len(text)) + text).hex()}
            initial = iter([
                json.dumps({"id": "aaaaa-aa", "module_hash": "0x" + smoke.sha256(b"\0asm\x01\0\0\0")}),
                json.dumps({"value": (smoke.PROJECT / "probe.did").read_text()}),
                json.dumps({"value": '{"schema_version": 1}'}),
                json.dumps(reply),
            ])

            def command(*args, **_kwargs):
                if args[:1] == ("build",):
                    return None
                if args[:4] == ("canister", "call", "governance-probe", "report") and args[4] == '("metrics")':
                    saved = json.loads(output.read_text())
                    self.assertEqual(saved["phase"], "collecting_metrics")
                    self.assertEqual(saved["reports"]["economics"]["report"], self.payload["report"])
                    return '{"response_bytes": "00"}'
                return next(initial)

            with patch.object(smoke, "ARTIFACTS", artifacts), patch.object(smoke, "icp", side_effect=command):
                with self.assertRaises(ValueError) as raised:
                    smoke.run_attempt("mainnet-smoke", "governance-probe", {"status": "running"}, output)
            saved = json.loads(output.read_text())
            self.assertEqual(saved["status"], "failed")
            self.assertEqual(saved["failed_phase"], "validating_metrics")
            self.assertEqual(saved["error"], str(raised.exception))
            self.assertIn(raised.exception.__cause__.stderr.strip(), saved["error"])
            self.assertEqual(saved["reports"]["metrics"], {"response": {"response_bytes": "00"}})
            self.assertIn("report", saved["reports"]["economics"])


class LifecycleTests(unittest.TestCase):
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
                        with self.assertRaises((RuntimeError, subprocess.TimeoutExpired)):
                            smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
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
                    return '{}'
                if args[:2] == ("network", "stop"):
                    self.assertEqual(json.loads(output.read_text())["status"], "running")

            with patch.object(smoke, "icp", side_effect=command), patch.object(smoke, "verify"):
                smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
            saved = json.loads(output.read_text())
            self.assertEqual(saved["status"], "passed")
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
        smoke.run(sys.executable, "-c",
                  "import os,signal,sys; from pathlib import Path; "
                  "Path(sys.argv[1]).write_text(str(os.getpid())); signal.pause()",
                  str(directory / "child"))
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
                        child = int((path / "child").read_text())
                        process.send_signal(signum)
                        wait_for_file(path / "stopped", process)
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
                    return '{}'
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
                    return '{}'

            with patch.object(smoke, "icp", side_effect=command) as commands, \
                    patch.object(smoke, "verify"), patch.object(smoke, "save_receipt", side_effect=checkpoint):
                with self.assertRaisesRegex(OSError, "receipt storage unavailable"):
                    smoke.run_attempt("local", "probe", {"schema_version": 1, "status": "running"}, output)
            self.assertEqual(commands.call_args.args, ("network", "stop", "local"))
            saved = json.loads(output.read_text())
            self.assertEqual(saved["status"], "failed")
            self.assertEqual(saved["network_cleanup"], "stopped")

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
