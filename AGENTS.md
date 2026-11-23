# Verification and handoff

Run `python3 scripts/suggest-checks.py --base <base-commit>` for focused feedback. The report explains which paths selected each check. CI continues to run the full checks.

At handoff, identify the commit tested, the behaviour observed, and the exact command and result or captioned artifact. Name unresolved review findings and the next required check. Label browser fixtures, scripted native runs and genuine-provider runs explicitly so the next reviewer can tell what each demonstrates.

# UI work

Follow [Topics design system](docs/design-system.md) and its linked screen specifications for all UI changes. Reuse its tokens, components and patterns; if a feature needs a missing token, component, state or pattern, extend the system and its documentation/examples in the same change before or alongside implementation.

Use synthetic temporary notes and an isolated absolute `TOPICS_APP_DATA_DIR` for tests. Never follow the repository `notes` symlink or use personal notes. See [contributing](docs/contributing.md) for project and verification details.
