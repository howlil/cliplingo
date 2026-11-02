# ClipLingo — Engineering Agent Rules

## Stack
- Windows 10/11 native application; C++20; CMake; Win32 shell.
- UI Automation, translation service, persistence, and Slint UI are planned, **not implemented**.
- Browser HTML reference at `design/reference/cliplingo-v6.html` is a demo, not runtime code.

## Commands
- Configure (VS 2022 on Windows): `cmake --preset windows-msvc`
- Build: `cmake --build --preset windows-msvc`
- Run: `out/build/windows-msvc/Debug/cliplingo.exe`
- Tests: TBD (no test suite in bootstrap; add risk-based proof with first behavior slice)
- CI: not configured at bootstrap; do not add repetitive or overlapping workflows.

## Structure / Boundaries
- `src/` is production native code; `design/reference/` is non-shipping visual reference.
- `.agents/product.md` owns product requirements and acceptance.
- `.agents/engineering.md` owns architecture/state/boundaries and verification stages.
- `.agents/DESIGN.md` owns UI tokens, interaction contract, and visual defaults.
- UI components must not directly own OS capture, provider credentials, translation requests, or SQLite data.
- Never ship HTML mock values, demonstration navigation, or macOS traffic-light chrome as Windows behavior.
- Native popup must preserve source focus except on explicit interaction; verify on real Windows desktop.
- Keep the application a modular monolith unless evidence requires another process. Do not add a service/framework because it is convenient.

## Quality and safety
- One complete narrow vertical slice at a time; prefer one active PR; avoid unrelated refactors and redundant CI.
- Inspect code and required tests before implementation; review the final diff and verify evidence.
- Focus on critical success/failure/edge paths; don't introduce ceremonial tests.
- No plaintext credentials or copied selected text in logs, diagnostics, or history.
- Respect consent, exclusion policy, and privacy before any outbound request.
- Do not claim native features work when only the HTML reference has been exercised.
- Ask before destructive migration, force-push, release or publish.
