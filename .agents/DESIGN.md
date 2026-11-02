---
platform: "Windows desktop; visual language adapted from macOS 27 (not native AppKit)"
colors:
  semantic: { text: "label", secondary: "secondaryLabel", tertiary: "tertiaryLabel", window: "windowBackground", control: "controlBackground", separator: "separator", accent: "controlAccent" }
  preview-light: { canvas: "#E7EAF0", window: "#FFFFFF", sidebar: "#F5F6F9", text: "#1D1D1F", secondary: "#63636B", accent: "#007AFF" }
  preview-dark: { canvas: "#1F2026", window: "#27282D", sidebar: "#303137", text: "#F2F2F5", secondary: "#B5B5BF", accent: "#69A7FF" }
typography:
  ui: { family: "system-ui; SF Pro only on supported Apple systems, Segoe UI on Windows", size: "13px", weight: 400 }
  popup-result: { size: "20px word/sentence, 14px paragraph", line-height: "1.45-1.7" }
  main-title: { size: "24px", weight: 650 }
spacing: { 1: "4px", 2: "8px", 3: "12px", 4: "16px", 5: "20px", 6: "24px", 8: "32px" }
radius: { small: "7px", control: "9px", card: "12px", popup: "15px", window: "16px" }
motion: { quick: "150ms", state: "190ms", easing: "cubic-bezier(.22,1,.36,1)", reduced-motion: "off" }
materials: { navigation: "subtle translucent only", content: "solid", low-resource: "no continuous blur animation" }
---
# ClipLingo Design

## Direction
- Product: A Windows selection translator that allows reading a contextual meaning and retaining vocabulary without switching applications.
- Character: quiet, precise, reading-oriented; no marketing language in utility screens.
- World: an annotated document margin inside a desktop reading workflow; prioritize text and selection over app chrome.
- Moment: a translated phrase is dominant; the contextual sense appears only when ambiguity matters.
- Signature: show the *meaning in this context* directly alongside the original sentence, not an AI chat or dictionary wall.
- Adapt macOS 27's hierarchy, semantic color and toolbar restraint, **not** a guaranteed replica of Liquid Glass on Windows. Never bundle SF Pro or Apple fonts; use installed system fonts.

## Surfaces / graph
- U1 Translation popup: **independent ephemeral window**, opened by M1 (B1-B3) hotkey in a compatible source app; never a tab in the main app. Loading (V1), result word (V2), sentence (V3), scrollable paragraph (V4), no selection (V5), provider error (V6), first-use consent (V7), protected source (V8), missing context (V9), duplicate save (V10), stale generation suppression (V11) and geometry fallback (V12). M2 (B5) copy, M3 (B6) save selected term, M4 dismiss/Escape. Popup appears without passive focus theft; interactive action must provide intentional keyboard entry and focus restoration.
- U2 Vocabulary: **main window and first full app view**. Open from tray or popup's Vocabulary action. Two-column at desktop width: searchable list on left, selected detail on right. M5 (B6) search/select; M6 remove with recoverable acknowledgement. Variant V13 empty collection; V14 no search matches. Content: saved term, contextual translation, original sentence, optional short sense note. No dashboard, translation history or unnecessary counters.
- U3 Settings: **secondary screen within U2's window**, reached by gear in vocabulary toolbar; M7 (B7) opens and Back returns to U2; no permanent nav item/tab. Group by Translation, Popup, Privacy. Conflict (V15) retains last valid shortcut; changing online consent updates eligibility of future outbound requests. Settings are reachable even when Vocabulary is empty.
- U4 Preview-only inspector: switches demo between U1 and U2, sample input, mocked response state, light/dark. It does **not** exist in production navigation. Route-like internal keys: `vocabulary`, `settings`; U1 is ephemeral and not route-addressable.

## State ownership
- ST1 capture/request generation and validity: workflow / session coordinator (engineering S1); UI never owns the authoritative result.
- ST2 popup presentation status: projection of ST1 and user dismiss; scroll/expansion UI-local, reset for a new generation.
- ST3 vocabulary: authoritative K6/K7 local storage; query and selected entry UI-local; filters derived, not duplicated.
- ST4 preferences and consent: K8 authoritative; editable settings form local; invalid hotkey does not replace saved binding.
- ST5 temporary preview interaction: demo local JS only; mock vocabulary and preferences reset when reloading preview.

## Patterns
- Window header: 46px unified toolbar with title and **one gear icon**; Settings replaces content in the same window and has an explicit Back action.
- Vocabulary: cardinality-aware composition: 0 entries instructional full canvas; 1 entry single reading canvas; ordinary collection selected term in a split list/detail; 500+ entries wider searchable list with two-column rows and a compact detail inspector; narrow window list → detail. Read `## Responsive collection and density` for geometry.
- Popup: nominal 392px, user setting 320-520px, bounded max-height with internally scrollable content. Word/sentence use larger translation type; paragraph uses smaller multiline reading type. Optional contextual sense is a quiet inline annotation, without a separate card.
- Use semantic roles; no raw CSS colors in production components. React/Slint implementation should map platform semantic tokens centrally, not scatter hex values. Glass-like material only for navigation/controls; do not blur the content layer.
- Do not replicate macOS traffic-light window buttons in Windows production; they are **illustrative only** in HTML preview. Use native Windows chrome/control semantics where needed.

## Composition, density and motion (v5)
- Retain the approved semantic Apple-inspired color, type, spacing, and radius vocabulary. Avoid card shells, nested rounded blocks, full-border vocabulary rows, decorative labels, and toolbar badges. Content sits on a solid reading plane; only app/window boundary and necessary area separators remain.
- Vocabulary is a flat two-pane reading workspace (~960×540px preview, resizable). The unified 48px toolbar includes Settings gear. Standard list pane ~290px, with compact search (35px), result count and Recent/A–Z sort. Rows ~51px show term + contextual meaning; selection uses a tinted flat row and 3px leading line, not cards. No footer labels with no user task.
- Detail hierarchy: language pair (useful metadata) → large source term → semantic-accent contextual meaning (no rounded card or filled callout) → one hairline → source sentence highlighting selected term → one concise explanatory sentence when disambiguation exists. Actions Copy and Remove remain visible in the top right; no history dashboard or duplicate labels.
- Cardinality: 0 → task-based empty with one Try translation action; 1 → one focused detail surface; 2–500 → 290px list + detail; >500 → wider dense table-like list (~55% width, ~38px rows, aligned term/meaning) + reading inspector. Virtualize the list with bounded visible rows and overscan. Use indexed/paged data access when implementing real large datasets.
- Search: filter term, meaning, source sentence; preserve focus and caret. No matches → in-list message and Clear search; do not show stale selected details. Sorting Recent/A–Z keeps selection meaningful. Large/long terms truncate in list but wrap in detail; long source paragraphs scroll within detail. At narrow width, use list → detail with an explicit Back and preserve the search query.
- Popup is an independent ephemeral surface (not a library tab); native geometry nominal 392px, constrained 320–520px and bounded scroll. Body contains source, translated meaning, optional inline sense; bottom actions Copy and Save, one low-emphasis Vocabulary affordance. Consent, no selection, loading, provider error and saved states retain explicit recovery moves.
- Motion is semantic and bounded: row-selection detail content crossfade + 7px rise (~190ms); Settings enters with ~13px horizontal translation (~190ms); popup rises 8px and scales from 0.985 (~190ms); toast/Undo enters 10px (~190ms); buttons react with color (~150ms). A moving progress line is allowed only while translation is pending. No looping idle animations, bounces, decorative parallax, blur animation, or transition-all. Never depend on animation to convey correctness; honor prefers-reduced-motion.
- No duplicate state ownership: vocabulary source of truth remains K6/K7, current item and search UI-local, popup result driven by the session generation (K3); new result must never replay a stale translation. Demo selectors for cardinality/popup samples are preview-only and must not ship in the native app.
- Windows implementation must validate native non-activating popup focus, DPI placement, keyboard entry to interactive actions, paragraph scrolling, and memory; browser preview is not proof of any OS behavior.

## Interaction, copy and accessibility
- Avoid descriptive filler such as "Welcome to ClipLingo". Show word, its meaning, actual saved sentence and specific actions (`Copy`, `Save word`, `Remove from vocabulary`).
- Empty collection: instructions explaining select → hotkey → Save; no mock vocabulary presented as real data. Error: state what failed and how to recover; selected text preserved for retry.
- No action hidden behind hover; mouse and keyboard reach every Settings/Vocabulary action. Target ≥24x24px, aim ≥32px for interactive controls. Visible focus; contrast WCAG 2.2 AA targets; accessible role/status for delayed responses.
- Escape dismisses U1; Settings Back returns to Vocabulary and restores sensible focus; passive U1 does not steal source focus. Dark/light and reduced motion, reduce transparency must preserve text legibility.
- No OS-level focus, hotkey, UIA, security, clipboard or translator semantics are proven by the HTML demonstration. They need native acceptance testing.

## UX gates / assumptions
- UG1: user can reach and inspect a saved term from opening app with no intermediate home/dashboard navigation.
- UG2: user can reach Settings only via U2 gear (or external OS tray entry); Back leads to Vocabulary. No prominent Settings tab.
- UG3: every popup failure in X1-X10 has a truthful visible state and retry/manual recovery where appropriate.
- UG4: word/sentence/paragraph readable in 320-520px popup, long paragraph scrolls without clipping close and actions.
- UG5: visual system adapts to light/dark and user accent preference; no heavy continuous translucency; never bundle Apple-proprietary font files.
- A13 [non-blocking]: macOS 27 aesthetic adopted for visual direction only; source product remains Windows-native in behavior and supported interactions.
- A14 [non-blocking]: all content in HTML prototype is illustrative, including word meanings and settings persistence; runtime state resets on reload.