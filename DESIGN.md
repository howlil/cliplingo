# ClipLingo Design Rules

ClipLingo is a Windows desktop utility. Its interface should communicate state and translation content with minimal chrome. Product task and information hierarchy take priority over decorative visual style.

## Product hierarchy

1. Valid selected text.
2. Indonesian translation result.
3. Translation route/status.
4. Model/settings controls.
5. Product branding.

The translation result is the primary visual element. Source text is secondary context. Branding must not displace task information.

## Translation popup

- Compact floating utility surface near the selection/cursor.
- Frameless and draggable from a small semantic route header such as `EN → ID`.
- Modest border/elevation only to separate it from the underlying application.
- Restrained radius.
- Native light/dark adaptation through system colors.
- No popup is visible before a valid selection is captured.
- New translations re-anchor automatically; manual drag only affects the current result position.

Do not use decorative glass blur, glow, gradients, oversized shadows, oversized radii, hero typography, redundant product-name headers, nested cards, bento layouts, or AI-themed visual effects.

## Settings

Settings is a control surface for a background utility, not a dashboard.

Use simple rows and dividers for:

- current translation route;
- shortcut;
- offline model state/install/remove;
- running status;
- explicit Quit action.

Do not add sidebar navigation, analytics, marketing copy, card grids, account surfaces, language marketplaces, or decorative illustrations without a real product requirement.

## Interaction

- System tray is the persistent application home.
- Left-click tray opens Settings.
- Right-click tray exposes compact Settings/Quit actions.
- Closing Settings keeps ClipLingo running.
- `Ctrl+Alt+T` acts only on a valid current text selection.
- No selection means no visible feedback; errors must not be manufactured from an absent user action target.
- Escape/close dismisses the translation surface without terminating ClipLingo.

## Quality test

Before accepting a UI change, ask:

- Does it make the translation task clearer or faster?
- Does it match Windows background-utility behavior?
- Is every visible element carrying information or enabling an action?
- Could the same UI have been pasted into an unrelated AI SaaS product? If yes, remove generic treatment and restore product-specific hierarchy.
