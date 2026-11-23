# Appearance and reading surfaces

Topic Detail uses one canvas from the topic name through the current understanding and revision history. The sidebar touches the window edge. Space groups the predicate, value, uncertainty, provenance and decisions for one claim. A divider marks a change of activity. A muted fill identifies quoted evidence. Fields retain visible boundaries and keyboard focus has a distinct ring. Floating BoardUI menus and calendars can use a border and shadow.

The Appearance control is available in the sidebar before connecting a folder. Follow system reacts to OS changes. Light and Dark persist across launches on this device. This preference belongs to the window's local storage, outside the notes folder.

## Adding a palette

Register its unique ID, visible label and base scheme (`light` or `dark`) in `apps/desktop/src/Appearance.tsx`. The shell sets both attributes on the HTML element so portal content inherits the palette. The base scheme supplies every semantic role and the CSS `color-scheme` used by native controls. In `tokens.css`, place overrides after the base palettes. For example, a future autumn palette based on Dark could set the following roles.

```css
:root[data-theme="autumn"] {
  --color-background-action: #845124;
  --color-background-action-hover: #73431e;
  --color-text-accent: #efc29d;
}
```

Components continue to use the existing semantic tokens. Preserve the meaning of warnings, privacy and approval. Check the palette's text, control boundaries, selected state, focus and open BoardUI overlays against their actual surfaces. Verify each new palette alongside the stored preference and live system-mode tests in `appearance.spec.ts`.
