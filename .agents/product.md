# Product — ClipLingo (Windows)

Status: research contract; no code changes or release claims. 2026-11-03.

## Problem
Actor/Acceptor: Windows user who regularly reads content in another language; project owner accepts the result.
Trigger: A word, sentence, or paragraph appears in another application and the user needs to understand it without losing their place.
Current behavior: Manually copy content into a translator and navigate back; context and vocabulary capture are disconnected.
J1: When I encounter unfamiliar text while using a Windows application, I want to understand its intended meaning in place and retain useful vocabulary, so I can continue my work without interrupting my flow.

## Behavior
B1: User selects a word, sentence, or paragraph in a compatible application and invokes a configurable translation shortcut.
B2: ClipLingo attempts to obtain the selected text without changing the source application's content, and with minimal disruption to focus and clipboard.
B3: A compact popup near the selection or cursor promptly appears with a meaningful loading state, without stealing keyboard focus on passive display.
B4: ClipLingo translates the selected text to the configured target language using available, explicitly permitted context; preserves meaning, handles word/sentence/paragraph layouts, and distinguishes alternatives from certainty.
B5: User reads the translation, optionally copies it, and dismisses the popup or switches back to the source application.
B6: User can explicitly save a word or phrase and view/remove saved vocabulary in a dedicated window.
B7: User can configure source/target languages, shortcut, popup dimensions/text size, translation provider/privacy mode, and exclusions in a settings window; settings persist across restart.
B8: Repeated invocation for a new selection replaces outdated responses; old requests must not overwrite newer results.

## Material Breaks
X1 (B2): No accessible selection -> clear, non-blocking 'No text selected or unsupported app' result; allow manual paste as recovery.
X2 (B2): Accessibility API unavailable or slow -> bounded attempt, then user-approved Ctrl+C fallback where safe; no silent replacement of an existing rich clipboard.
X3 (B2): Secure/password content, elevated process, UAC/secure desktop, protected app -> do not bypass protection; display unavailable state.
X4 (B3): Selection geometry missing, out-of-screen, negative coordinates, multi-display/DPI -> clamp popup to current display's usable work area and anchor near cursor.
X5 (B4): No network/provider credentials, timeout, quota, or unsupported language -> actionable retry/configuration state; never invent translation.
X6 (B4): Context missing or language ambiguous -> translate selected text without claiming unavailable context; optionally show alternative meanings.
X7 (B6): Duplicate vocabulary -> no duplicate record for identical normalized source, language pair, and sense, with visible feedback.
X8 (B7): Shortcut is already registered by another application -> show conflict, keep the last working shortcut, and allow reassignment.
X9 (B8): Rapid re-trigger -> newest selection wins and stale output is discarded.
X10 (B4/B7): First use of online provider -> explicit disclosure and consent before any selected text is transmitted externally; app exclusions honored.

## Constraints
C1: Windows 10/11 desktop; no administrator rights required for normal operation.
C2: Respect Windows UI Automation/provider availability and OS integrity boundaries; 'all apps' is a best-effort compatibility goal, not a guaranteed capability.
C3: Low idle CPU and memory, prompt popup feedback, no continual OCR or clipboard surveillance.
C4: Fast, declarative UI iteration is desirable; no mandatory browser engine on the always-on translation path.
C5: Translation privacy: collect only the selected text and the minimum accessible context; send externally only under user-approved online mode; saved vocabulary stays local by default.
C6: App works as a tray/background utility; translation popup must not navigate users away from their original context.
C7: Saved vocabulary and settings persist locally; raw unsaved selection is not saved to a history by default.

## Quality Bars — PROPOSED, not measured
Q1: Hotkey-to-visible-popup/loading indication p95 <= 150 ms on an agreed Windows reference machine.
Q2: Hotkey-to-final translation p95 <= 2.0 s for short text and <= 4.0 s for a 300–500-word paragraph, when cloud provider and network meet an agreed baseline; failure states remain responsive.
Q3: Idle app private working set <= 80 MiB, core/tray/popup combined, settings closed, excluding optional offline translation models; idle CPU average <= 1% after startup. Test and adjust by measured feasibility.
Q4: No lost original text, unintended paste, persistent focus theft, or clipboard damage in the tested application matrix.
Q5: At least 90% meaningful, context-appropriate results on a pre-agreed bilingual evaluation set; human-scored and broken down by isolated terms, ambiguous sentences, and paragraphs. Subject to feasibility baseline.
Q6: Target coverage: a reference set including Chrome/Edge, Notepad, Word, VS Code, and a selectable-text PDF; log actual success separately for UIA-only vs copy fallback. No 100% coverage promise.

## Scope
In: explicit shortcut-to-translate; configurable source/target (EN/ID validation first); accessible text selection and safe fallback; contextual translation where available; responsive popup for word/sentence/paragraph; manual copy; save/remove/list vocabulary; settings and app exclusions; local persistence; privacy consent; honest failure and loading states.
Out: OCR/screenshot translation, translating arbitrary canvas or protected UIs, invasive global keyboard hooks by default, always-on automatic monitoring, offline models as a hard MVP requirement, multiple concurrent providers, grammar tutoring, speech, AI chat, sync, spaced repetition, browser extensions, auto-replacing selected text, and cloud account creation.

## Proof
P1 (B1,B2,X1,X3,Q6): Given a real selectable text field/document, invoking the shortcut obtains exact selected text or returns a nonintrusive actionable failure; protected/unsupported surfaces are never bypassed.
P2 (B2,X2,Q4): Given existing rich clipboard contents, fallback never overwrites them silently; aborting or failing leaves source text intact, and fallback consent is respected.
P3 (B3,X4,Q1): Given a selection on primary, scaled, secondary, and negative-coordinate monitors, the popup visibly opens within the current work area, without unsolicited focus changes, within measured UI feedback budget.
P4 (B4,X5,X6,Q2,Q5): Given word/sentence/paragraph samples, result uses only available context, is scored against reference meanings, and shows retry/provider error instead of fabricated results when unavailable; collect stage timings.
P5 (B5): Given a translation, user can read, copy intentionally, and dismiss it while returning to source workflow.
P6 (B6,X7): Given a saved vocabulary entry, after restart it is listed once, with correct source/target and available context, and can be removed.
P7 (B7,X8): Given changed languages/shortcut/size/provider/exclusions, behavior follows persisted settings after restart; collision rejects the new shortcut without breaking the old one.
P8 (B8,X9): Given two rapid selections with out-of-order replies, only the latest result appears.
P9 (X10,C5,C7): Given first online request with no consent, no text leaves the device; saved vocabulary stays local and excluded app text is not sent.
P10 (Q3): Five-minute idle trace records private working set and CPU; measure popups/settings open and closed separately, with offline models reported separately.

## Assumptions
A1 [non-blocking]: Explicit shortcut is the default trigger; auto-popup on mouse selection is deferred.
A2 [non-blocking]: `Ctrl+Alt+L` is a candidate default; `Ctrl+C` then `L` (while Ctrl remains held) is an optional researched alternative, requiring its own chord detection and conflict testing.
A3 [non-blocking]: EN<->ID is the first acceptance language pair; provider-supported pairs may be exposed after validation.
A4 [non-blocking]: Online provider is acceptable following explicit first-use consent; offline translation is a future mode with a different memory budget.
A5 [non-blocking]: Performance numbers Q1-Q5 are hypotheses for feasibility tests, not achieved metrics or vendor guarantees.

## Handoff / Open Research
- Engineering-design: prove UIA compatibility, safe fallback, focus and clipboard guarantees; choose UI toolkit only after an actual Windows no-activate/long-text/HiDPI spike.
- Design-graph: model three surfaces (Popup, Settings, Vocabulary) and loading, empty, success, paragraph, error states.
- Test-engineering: benchmark stage timings and working-set/CPU on a repeatable machine and mixed application matrix.