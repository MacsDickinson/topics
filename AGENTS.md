# UI work

Follow [Topics design system](docs/design-system.md) and its linked screen specifications for all UI changes. Reuse its tokens, components and patterns; if a feature needs a missing token, component, state or pattern, extend the system and its documentation/examples in the same change before or alongside implementation.

Use synthetic temporary notes and an isolated absolute `TOPICS_APP_DATA_DIR` for tests. Never follow the repository `notes` symlink or use personal notes. See [contributing](docs/contributing.md) for project and verification details.

# Documentation

Documentation is mandatory. Every change or new feature must update the docs in the same pull request: the docs site in [`docs/site/`](docs/site/README.md) (architecture, user guide, feature breakdown, milestones, known limits), plus `README.md`, `docs/*.md` and `features/*/PLAN.md` where relevant. UI changes need fresh screenshots from synthetic notes only. The pull request description must list the docs updated, or say "Docs: no user-visible or architectural change" with a reason.

Follow the standards, update map and reviewer checklist in [`.claude/skills/documentation/SKILL.md`](.claude/skills/documentation/SKILL.md). The site deploys to GitHub Pages from `main` through `.github/workflows/docs.yml`.
