# Host #5 implementation evidence

The Rust process integration candidate is WITHDRAWN. Do not apply its patch or treat its passing preliminary fixtures as acceptance. It fails the current successful background-runtime handoff contract, as recorded in background-handoff-candidate.log.

The candidate source files and rejected-consumer-integration.patch are diagnostic evidence only. The patch includes the pre-existing maintainer-selected Host 0.5.0 dependency update against Query HEAD 08012b4bedd288b5882c0d8134ea94aa739228f6; those dependency changes are not a new process adoption. The untracked Rust example is preserved separately as governance_process.rs.

The retained Query repair preserves original operation/network-cleanup exceptions through receipt failures, retains available subprocess diagnostics, and leaves the current process and background startup implementation in place. consumer-receipt-tests-final.log records all 33 receipt/process tests passing on Linux. The selected cache was explicitly prepared with cargo fetch --locked --offline; the production library and CLI dependency trees have no ic-host-process dependency. Snapshot (59 files), declarations/inheritance, local documentation links (203 references across 33 documents), changelog and git diff checks pass.

Native consumer CI, live network smoke, fresh MSRV, full CI and release gates were not run. No sibling source, package version, index, commit, tag or release ref was changed. The Host issue remains open:
https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6055899595
