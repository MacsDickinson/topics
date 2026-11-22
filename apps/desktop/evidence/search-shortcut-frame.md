# Search shortcut frame timing

Hosted desktop run 42562315254 failed the immediate selection assertion with `[6, 6]` instead of `[0, 6]`. The shortcut schedules input focus and selection with `requestAnimationFrame` in main.tsx.

The recorded RED holds that frame during the original synchronous assertion and reproduces the same `[6, 6]` mismatch. The revised test verifies the pre-frame caret, releases the real queued frame, and polls for the complete `[0, 6]` selection and focused search input; no product behavior or timeout changed.

Focused Chromium test: 10/10 passed with a controlled delayed frame. Full Chromium suite: 114/114 passed, and the desktop TypeScript/Vite build passed; command outputs are adjacent.
