# Certificate boundary fixtures

Copied unchanged from ic-platform-new commit
`126479e64adfc13fa21aa217caa19a298bcdc19e`,
`crates/ic-platform/tests/fixtures/`: frozen PocketIC root/delegated observations
and their simulator root key. These are public test data, not mainnet trust inputs.
The supplier owns their acquisition/protocol proofs. Query checks that its actual
Agent certificate identity crosses the local borrowed projection, retaining exact
time and refusing wrong authority/signatures. No production codec is copied.
