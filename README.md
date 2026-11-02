# ClipLingo

Native Windows contextual translator, designed for selecting text from another application and displaying a compact translation popup.

**Status:** repository initialization. The translation engine, text capture, and native UI are not yet implemented.

## Project references

- [Product contract](.agents/product.md)
- [Engineering contract](.agents/engineering.md)
- [Design rules](.agents/DESIGN.md)
- [Interactive design reference v6](design/reference/cliplingo-v6.html) — standalone HTML; open locally in a browser. This is a **prototype**, not the production UI.

## Build (Windows)

Requires Visual Studio 2022 with Desktop development with C++ and CMake 3.21 or newer.

```powershell
cmake --preset windows-msvc
cmake --build --preset windows-msvc
```

Run `out/build/windows-msvc/Debug/cliplingo.exe`.

Unit proof (CTest):

```powershell
ctest --test-dir out/build/windows-msvc -C Debug --output-on-failure
```

`core_unit` covers the request generation gate and the consent/exclusion policy; `layer_boundaries`
enforces the include boundary from `.agents/engineering.md` (F1, F3, F4).

This executable is an initial Win32 shell only. See `.agents/engineering.md` for the planned Win32/UI Automation capture boundary and proposed UI toolkit. Slint remains a candidate pending native popup feasibility testing.

## Boundaries

- Target Windows 10/11; C++20, Win32 native integration.
- Single-process modular monolith when the product workflow is implemented.
- Preserve the vocabulary-first main window and transient translation popup defined by design v6.
- Do not treat the HTML demonstration, simulated translations, or macOS-style chrome in it as production behavior.
- No automatic CI or release workflow at initialization; introduce proportionate verification with implementation.
