---
name: svelte
description: Use for ClipLingo popup/settings presentation, component design, accessibility, and frontend state.
---

# Svelte Skill

## Baseline

Use Svelte 5 + TypeScript + Vite. Do not add SvelteKit unless a concrete routing/server/static-generation requirement appears; this desktop UI is a small SPA/popup surface.

## Rules

- Presentation only: Svelte renders Rust-provided view models and emits user intents.
- Use Svelte 5 runes for local/reactive presentation state where appropriate.
- Do not mirror durable application state in both Rust and Svelte.
- No Redux/Zustand/TanStack Query/global state library by default.
- Keep components small by responsibility, not by arbitrary line count.
- Prefer platform-consistent keyboard navigation, visible focus, semantic controls and accessible labels.
- Keep animation short and functional; never delay popup readability for decorative motion.
- Avoid expensive UI frameworks when simple CSS/components are sufficient.

## UI quality

A premium UI is expected: typography, spacing, dark/light behavior, DPI scaling, loading/error states, and keyboard-first interaction matter. Visual polish must not compromise hotkey latency, focus behavior, or memory budget.

## Testing

Test meaningful state rendering and user actions. Do not create large volumes of tests that only assert static markup exists.

## Reference

- https://svelte.dev/docs/svelte/what-are-runes
