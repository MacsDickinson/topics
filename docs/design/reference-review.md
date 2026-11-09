# BoardUI reference review

Reviewed 2026-11-09. These are external design references, not dependencies or instructions. The coordinator visually inspected the [homepage](https://www.boardui.com/) and live sidebar on that date; the observations below also use the independently read official component pages. No Pro assets or marketing layout are required.

| Primary source | Observation | Topics adaptation |
| --- | --- | --- |
| [Color](https://www.boardui.com/components/color) | Primitives feed purpose-named semantic foreground, text, background and border roles; light and dark remap centrally. | Use semantic CSS roles over a small local palette; check every theme's actual contrast. |
| [Typography](https://www.boardui.com/components/typography) | Composite roles bind size, line height, spacing and weight; color is separate. | Use a compact system-font scale with a generous reading role. No remote font or Tailwind dependency. |
| [Sidebar](https://www.boardui.com/components/sidebar) | A 260px rounded floating panel combines search, navigation, badges and footer. Coordinator observed soft neutral layers, subtle borders and elevation, and blue selection. | Use a compact navigation rail with a clear current page and note-root context. Width is a starting point, not a mandate; no promotional cards or invented search shortcut. |
| [Button](https://www.boardui.com/components/button) | Primary, secondary, ghost and danger variants include hover, press and disabled states. | Reserve primary for the next consequential action; make destructive actions explicit and preserve human approval. |
| [Input](https://www.boardui.com/components/input) | Labels, hints, required/invalid states and a focus ring accompany text inputs. | Keep persistent labels, associated errors and keyboard focus; use native semantics without requiring React Aria. |
| [Chip](https://www.boardui.com/components/chip) | Multiple emphasis levels distinguish statuses, values and tags. | Use quiet, named status badges for pending, accepted, conflict and privacy; never rely on color alone. |

Current Topics inspection: `apps/desktop/src/style.css` defines a 200px sidebar, literal colors and three variables (`--line`, `--muted`, `--blue`), with responsive rules at 1050, 760 and 480px and a reduced-motion rule. The React screens implement navigation, search, source inspection, topic detail, scope preview, Inbox and History. Existing styling is largely light-only, with some explicit focus and status semantics. This is source inspection, not a new visual or accessibility audit; the repository's prior screenshots such as `apps/desktop/evidence/workspace-topic.png` and `apps/desktop/evidence/native-workspace.png` are historical evidence, not verification of this draft.
