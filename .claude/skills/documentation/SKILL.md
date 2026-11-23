---
name: documentation
description: Keep Topics documentation current. Use whenever a change touches behaviour, UI, architecture, data formats, IPC commands, crates, MCP tools or features: decide which docs to update (docs/site, README.md, docs/*.md, features/*/PLAN.md), apply the documentation standards, and satisfy the documentation definition of done before finishing or opening a PR.
---

# Documentation standards

Every change or new feature must be documented in the same pull request. The docs site lives in `docs/site/` (see `docs/site/README.md`), deploys to GitHub Pages from `main`, and is the public description of how Topics works.

## What to update

| Change | Update |
| --- | --- |
| New feature or flow | A row in the **Feature breakdown** table (`#features`), a **User guide** section or steps (`#guide-*`), and **Architecture** if structure changed |
| UI change | Fresh screenshots in `docs/site/img/`, captured from synthetic notes only, plus any user-guide text they illustrate |
| New or changed IPC command, crate, data format or MCP tool | **Architecture** sections (`#system`, `#crates`, `#data`, `#cloud`); keep the stat counts (workspace members, IPC commands, model tools) correct |
| Milestone reached | **Milestones** timeline (`#milestones`) |
| New limitation or removed limitation | **Known limits** (`#limits`) |
| Any of the above | The relevant `docs/*.md` and `features/*/PLAN.md` notes, and `README.md` / `docs/contributing.md` where they describe the same thing |

Verify counts against the code rather than copying them: IPC commands are the `generate_handler!` list in `apps/desktop/src-tauri/src/main.rs`, crates are `members` in `Cargo.toml`, model tools are the tool list in `crates/topics-runtime/src/bridge.rs`.

## Standards

- Describe what the app does **now**, in plain present tense. No aspirational claims. Work that is not built is marked **Planned** (or "Gated on pilot" where that applies).
- Use Mermaid for flows and architecture (`<pre class="mermaid">` in the site, ```` ```mermaid ```` in Markdown).
- Use relative links and image paths only, so the site works from a Pages sub-path. Never `/absolute` paths.
- Screenshots come from **synthetic** notes with an isolated absolute `TOPICS_APP_DATA_DIR`. Never personal notes and never the repository `notes` symlink (see `AGENTS.md`).
- Give every image meaningful alt text describing what it shows.
- Keep `docs/site`, `README.md` and `docs/contributing.md` consistent; if a fact changes in one, fix it in all.
- Keep the `{{BUILD_SHA}}` / `{{BUILD_DATE}}` placeholders in the site; the workflow fills them.
- Preview with `python3 -m http.server -d docs/site` and run `python3 docs/site/check-links.py docs/site` before finishing.

## Definition of done

A change is not done until the docs reflect it in the same PR. The PR description must either list the docs updated, or say `Docs: no user-visible or architectural change` followed by the reason (for example, an internal refactor with identical behaviour). The PR template has a Docs section for this.

## Reviewer checklist

- [ ] The PR description lists docs updated, or gives a reasoned "no user-visible or architectural change".
- [ ] Feature table, user guide, architecture, milestones and known limits match the change, as applicable.
- [ ] Counts (crates, IPC commands, MCP tools) match the code.
- [ ] Text is present tense and describes current behaviour; unbuilt work is marked Planned.
- [ ] Diagrams are Mermaid and still match the code.
- [ ] Screenshots are fresh, synthetic-only, and have alt text.
- [ ] All links are relative and `check-links.py` passes.
- [ ] `README.md`, `docs/contributing.md` and `docs/site` agree.
