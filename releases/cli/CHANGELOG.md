# CLI and text presentation changelog

Covers `ic-query-cli` (`icq`) and `ic-query-text`. The CLI's existing finalized
baseline is 0.85.0; earlier coordinated releases remain in [shared history](../history.md).
The new text package starts at the same group baseline without changing existing
package versions or claiming an earlier publication.

Baseline version: `0.85.0`.

## [0.86.0]

### ⚠️ Breaking

- Import reusable report renderers from `ic-query-text`; the CLI consumes that
  package alongside Query's typed reports. Text presentation retains its existing
  output while JSON adopts the [Query 0.86 contract changes](../query/0.86.md).
- Release and publication commands require explicit `RELEASE_GROUP=cli` or
  `RELEASE_GROUP=query`. Each group owns its versions, notes and selected package
  batch within the existing workspace.

### 🧹 Cleanup

- Use library-owned SNS/ICRC choices and move report presentation to one owner.
  CLI arguments, process IO, progress and terminal policy remain in the CLI.

### ⬆️ Dependencies

- Adopt Build/Text 0.74, Foundation 0.74.1 and Platform 0.18 through the shared
  root catalog. Query owns report arithmetic without Metrics. Development fixtures adopt Wire
  0.15.0 with unchanged protocol bytes and budgets. Keep the full CI gate and use
  Shared Tooling 0.27.27 for independent release selection.

Details: [0.86 notes](0.86.md).

Inspect the selected group version before release:

```bash
make version RELEASE_GROUP=cli
```
