# ClipLingo CI strategy

CI is split by **risk**, not by the number of milestones. The five workflow
files represent different validation boundaries; they must not all run for
every file change.

| Workflow | Automatic trigger | Purpose |
| --- | --- | --- |
| `geometry.yml` | PR or master push changing native src/tests/CMake/this workflow | Fast MSVC host smoke + native CTest |
| `model-inference.yml` | PR changing model pack builder/verifier/this workflow | Real OPUS EN→ID model correctness |
| `native-full-build.yml` | PR changing native dependency interface / CMake / this workflow | Full native MSVC link |
| `ocr-native.yml` | PR changing OCR/region/CMake/this workflow | Tesseract-enabled build |
| `release.yml` | PR touching release scripts/version/tests/notes: **version tests only**; relevant product push to master: package; manual dispatch; daily recovery | Validate and publish prerelease after full native ZIP+model verification |

All long-running PR workflows cancel *superseded PR attempts*; a new code
commit does not cancel a master production release. Pushes on feature branches
do not duplicate PR validation.

**Automatic releases:** a push to `master` affecting `src/**`, `CMakeLists.txt`,
the model builder/verifier, release packager or release workflow creates a
candidate beta. The **model and OCR-enabled Windows portable ZIP** are built
only from the merged master source (or a manual dispatch); publishing occurs
only after testing the extracted executable with the exact offline model.
A daily UTC 02:17 fallback checks for an untagged release-worthy master commit;
documentation-only changes do not create beta versions.

**Manual:** use GitHub Actions → `Release portable Windows beta` → Run workflow
on master when an explicit release is necessary, or on a feature branch to
perform a dry-run package build without publishing. Native/OCR/model workflows
also retain manual dispatch to diagnose a dependency change.

**Cost control:** full CTranslate2, Tesseract/vcpkg, PyTorch model conversion
and artifact packaging are expensive. Avoid starting them on every PR commit,
trivial README edit or hourly timer. Workflow history remains visible in
GitHub even when new triggers are narrowed.

**Boundary:** only native compilation and release packaging are automated.
Physical Windows GUI/UAI capture, OCR accuracy, Narrator, real memory/latency
and installer acceptance still require separate manual verification.

**Latest-master wins:** superseding master pushes cancel stale in-flight release builds; release tag freshness gate prevents publishing a different commit than was tested. Manual workflow runs are not auto-cancelled.
