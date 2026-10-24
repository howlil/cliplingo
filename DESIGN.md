# ClipLingo — Approved Design Baseline

> Status: **approved product/UI baseline** (2026-10-25). Change behavior or visual intent only with explicit product/design approval. This is **Apple-inspired**, not an official Apple design-token export, and the implementation target is **native Windows**.

## 1. Product presentation

ClipLingo is a quiet Windows reading utility, not a chat app. The application resides in the **Windows notification area (system tray)**. Settings are opened from the tray. Translation appears as a **floating, text-only result above the cursor**. No persistent main window, web-like toolbar, dashboard, or source-text duplication.

### UI surface hierarchy

```text
Windows Desktop
├─ Source application (browser / PDF / editor / other)
├─ System tray icon
│  └─ ClipLingo tray menu
│     ├─ Pause / Resume
│     ├─ Capture screen text (on-demand; only when supported)
│     ├─ Settings…
│     └─ Quit
├─ Translation popup
│  └─ Translated TARGET TEXT ONLY
└─ Settings window (separate native HWND)
   ├─ Sidebar
   │  ├─ General
   │  ├─ Text Capture
   │  ├─ Translation Popup
   │  ├─ Appearance
   │  ├─ Language Models
   │  └─ Privacy
   └─ Detail pane (section header → grouped controls → contextual preview/status)
```

**Keep the chrome out of the result popup:** no header, language badge, copy, swap, pin, close, settings, retry or source-text controls in the popup. The demonstration's browser/PDF/editor and control panels are **simulation surfaces**, NOT parts of the shipped translation popup.

## 2. Design tokens

Values below are the **approved preview starting tokens**, not a pixel-perfect license to ignore Windows accessibility and DPI. Treat dimensions as **logical/density-independent pixels (DIPs)**, convert at Windows per-monitor DPI boundaries; never assume one device pixel per DIP.

### Color semantics

| Semantic role | Light | Dark | Use |
| --- | --- | --- | --- |
| Accent | `#007AFF` | `#0A84FF` | active navigation, focus, selected controls |
| Accent subtle | `#E8F2FF` | `#173451` | low-contrast selection backgrounds |
| Primary text | `#1D2430` | `#F5F7FC` | primary content and popup translation |
| Secondary text | `#68717F` | `#BBC3D0` | supporting information |
| Tertiary text | `#8E98A6` | `#8D97A8` | metadata and captions |
| Primary surface | `#FFFFFF` | `#202631` | settings cards/window |
| Secondary surface | `#F5F7FA` | `#272E3B` | sidebar and control groups |
| Subtle separator | `rgba(30,45,70,.105)` | `rgba(255,255,255,.105)` | separators, not decorative outlines |
| Success | `#259D59` | `#35C36B` | installed/ready |
| Danger | `#E24945` | `#FF6961` | actionable error |
| Popup solid | `#FFFFFF` | `#222934` | default high-legibility text surface |
| Popup glass (optional) | `rgba(255,255,255,.78)` | `rgba(31,38,48,.78)` | optional native blur where supported |

Support a **tinted** surface as a subtle accent mix; use solid when transparency, contrast, driver support or high-contrast accessibility is unsuitable. Do not reproduce artificial gradient decoration from demo wallpaper as the product's UI theme. These values are ClipLingo's own design system.

### Geometry, type, motion

| Token | Baseline |
| --- | --- |
| App font | `Segoe UI Variable`, fallback `Segoe UI` |
| Translation font | 15 DIP, regular, line height approx. 1.55–1.65 |
| Settings body | 13 DIP; supporting labels 11–12 DIP; heading ~19 DIP |
| Font weight | Regular 400 / medium 500 / semibold 600 |
| Spacing scale | 4 / 8 / 12 / 16 / 20 / 24 DIP |
| Button/control radius | 10 DIP |
| Group radius | 13–15 DIP |
| Popup radius | 15 DIP default, configurable |
| Popup padding | 14 DIP default, configurable |
| Popup maximum width | **420 DIP** default, configurable |
| Popup maximum height | **300 DIP** default, configurable |
| Popup anchor gap | **12 DIP** between cursor and popup |
| Settings desktop-window reference | roughly 848 × 624 DIP; sidebar roughly 196 DIP |
| Feedback animation | ~120 ms; reduce/disable with reduced-motion preferences |
| Iconography | coherent 16–18 DIP outline icons, ~1.8 DIP stroke, one icon language |

Use a native icon set with predictable silhouettes at 100%, 125%, 150%, 200% DPI; do not substitute emoji, random Unicode glyphs, characters resembling icons, or ambiguous metaphors. Windows control hit targets and keyboard navigation take priority over preview pixel values.

## 3. Popup layout contract

### Content-driven intrinsic sizing — NON-NEGOTIABLE

```text
translated target text
  → measure with final font + padding + DPI
  → intrinsic content width (do not stretch to max-width)
  → clamp to maximum width and available monitor area
  → wrap / remeasure content height
  → clamp to maximum height and available monitor area
  → overflow strategy for long content (readable scrolling/accessibility)
```

**Maximum width and maximum height are limits, not forced dimensions or minimum sizes.** A one-word result should look like a compact label-sized text surface, while a paragraph grows only as far as the configured limits. No arbitrary fixed width, fixed height, giant empty region, minimum container height, or truncation without a readable overflow path.

### Cursor placement

1. Capture cursor and foreground-app context **once at trigger time**; do not follow subsequent mouse motion.
2. Place popup horizontally centered on that frozen cursor anchor when possible.
3. Prefer popup **above** cursor with ~12 DIP spacing; fall below when upper space is insufficient.
4. Clamp geometry into the selected monitor **work area** (taskbar excluded). Support negative virtual-screen coordinates, multi-monitor and mixed-DPI.
5. Do not steal focus, enter normal Alt+Tab, or obscure selection unnecessarily.
6. Escape/dismiss follows user control policy; never show the popup for empty capture, failed inference, or stale request.

### Popup content and states

| State | Visual contract |
| --- | --- |
| Hidden/idle | No popup |
| Capture missing/invalid | No translation popup |
| Inference pending | No target-result popup yet; errors/status belong to tray/Settings, not inserted as fake translations |
| Success: word | Compact intrinsic rectangle containing only target text |
| Success: sentence | Content-wrapped text rectangle |
| Success: paragraph | Wrapped container up to max width/height, with accessible overflow |
| Fatal/unrecoverable failure | No success popup; surface actionable status outside the result popup |

## 4. Tray menu

- **Tray icon** is the primary application entry point, including Windows hidden notification icons (`^`).
- Native menu: Pause/Resume, Capture screen text (when available), Settings, Quit.
- Opening Settings is available even when capture or model is paused/not installed.
- Tray icon restores after Windows Explorer taskbar recreation.
- Never rely on a floating toolbar to expose basic settings.

## 5. Settings composition

Use a **single native Windows settings window** with sidebar and content pane. Keep grouped controls clear, compact, accessible, and not nested unnecessarily.

| Section | Component blocks | Meaningful states |
| --- | --- | --- |
| General | Startup, global shortcut (default `Ctrl+Alt+T`), app status | Running / Paused / shortcut conflict |
| Text Capture | Selection, hover scope, clipboard fallback opt-in, OCR region capture | Enabled / denied / unsupported |
| Translation Popup | Cursor anchor policy, max width/height, auto-dismiss | Auto-fit, within bounds; preview |
| Appearance | Theme, Solid/Glass/Tinted, padding, radius, font size | Contrast/high-contrast fallback |
| Language Models | EN→ID installation/status; ID→EN optional; model mode | Missing / downloading / validating / ready / failed |
| Privacy | Offline processing, excluded apps, content logging policy | Excluded / allowed / protected |

Control anatomy: section title → one brief description → flat grouped rows → label and optional one-line help → native switch, select, slider, or shortcut field → contextual status. **Only one authoritative copy of each preference.** The inline preview shows the actual popup shape/content and updates with settings; it is not a second settings editor.

## 6. Accessibility and interaction

- Clear focus order and visible keyboard focus in Settings.
- Native system font fallback; follow text scaling, high contrast, reduced motion, RTL/BiDi and Unicode shaping.
- No text baked into raster images. Expose translation via an accessible text/UIA representation even when visual popup does not take focus.
- Do not assume translucent backgrounds are always readable; fallback to solid where necessary.
- Provide reachable diagnostics for actionable failures in Settings/tray, not in the text-only result popup.
- Preserve keyboard-only access to Settings and an explicitly documented dismiss action.

## 7. Anti-drift & design acceptance

A change to the design must pass:
- One word renders intrinsically small; sentence/paragraph sizes grow only to maximums.
- Text-only popup contains **zero controls** and no source text.
- Cursor-anchored placement works at top/bottom edges, negative virtual-screen positions and per-monitor scaling.
- Tray → Settings navigation matches six-section hierarchy.
- No pseudo-Apple imported SF font, copied Apple assets, arbitrary glass decorations or gradients.
- Icons are semantic, sharp and consistent; settings are usable with keyboard/high contrast.
- Visual fidelity is judged against the approved **ClipLingo Windows Tray & Settings interactive preview** from the design discussion; this Markdown is the persistent implementation specification.

**Ownership:** `DESIGN.md` governs visual and interaction presentation; `.agents/product.md` governs desired behavior and scope; `.agents/engineering-design.md` governs technical boundaries. Architecture does not silently change this baseline.
