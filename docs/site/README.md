# Topics docs site

A static, dependency-free explainer of how Topics works. It is maintained here, deployed to GitHub Pages by `.github/workflows/docs.yml`, and must be updated in the same pull request as any change that affects behaviour, UI, architecture, data formats, commands or features (see `.claude/skills/documentation/SKILL.md`).

## Structure

```
docs/site/
├─ index.html       one page; sections are anchored and listed in the left nav
├─ style.css        all styling (light and dark via prefers-color-scheme)
├─ img/             screenshots used by the user guide
├─ check-links.py   relative link validator, also used by the workflow
└─ README.md        this file
```

All links and image paths are relative, so the site works from a Pages sub-path such as `https://<owner>.github.io/<repo>/`.

### Section layout

| Group | Sections (anchor ids) |
| --- | --- |
| Overview | At a glance (`at-a-glance`), Core principles (`principles`) |
| **Architecture** | System map (`system`), Crates & layers (`crates`), Data model & storage (`data`), The cloud boundary (`cloud`), Recoverable writes (`writes`) |
| **User guide** | `guide-start`, `guide-setup`, `guide-review`, `guide-topics`, `guide-today`, `guide-more` |
| **Product** | Feature breakdown (`features`), Milestones (`milestones`), Known limits (`limits`) |

When you add or rename a section, update the left `<nav class="toc">` too.

### Diagrams

Diagrams are Mermaid (`<pre class="mermaid">`), rendered in the browser from the jsDelivr CDN (`mermaid@11`). Mermaid is not vendored, to keep the repository small. If the CDN is unreachable or JavaScript is off, a note above the content explains that the diagrams need JavaScript and access to `cdn.jsdelivr.net`; the surrounding text describes each flow.

### Build stamp

The footer and nav contain `{{BUILD_SHA}}` and `{{BUILD_DATE}}`. The workflow replaces them in the published copy only; a local preview shows the literal placeholders. Do not remove them.

## Preview locally

```sh
python3 -m http.server -d docs/site 8000
# open http://localhost:8000/
```

Mermaid needs network access to the CDN for diagrams to render.

## Screenshots

Screenshots live in `img/` and are referenced as `img/<name>.png` with descriptive alt text.

- Capture only from **synthetic** notes, with an isolated absolute `TOPICS_APP_DATA_DIR`. Never use personal notes or follow the repository `notes` symlink (see `AGENTS.md`).
- Existing images were copied from `apps/desktop/evidence/` (Playwright captures such as `apps/desktop/evidence/design/*.png`, and native captures). Regenerate them with the desktop verification flow in `apps/desktop/README.md` and `docs/contributing.md`, then copy the result into `img/`.
- Take them at 1x, keep them under about 250 KB, and reuse the file name when replacing a picture so links stay valid.

## Checks

The workflow builds the site into `_site/` and fails if any relative `src`/`href` (including `../` escapes) points at a missing file or outside the site. Run the same check locally:

```sh
python3 docs/site/check-links.py docs/site
```

## Deployment

`.github/workflows/docs.yml` runs the build and link check on pull requests, and deploys `_site/` to the `gh-pages` branch on pushes to `main` (or manual dispatch from `main`). Set Settings → Pages → Source: "Deploy from a branch", `gh-pages` / root once.

GitHub does not start a branch Pages build for pushes made with the workflow's `GITHUB_TOKEN`, so the deploy job requires a repository secret `DOCS_DEPLOY_TOKEN`: a fine-grained personal access token limited to this repository with Contents: read and write. The job fails with a clear error when the secret is missing, so a stale live site cannot go unnoticed. The token also creates the `gh-pages` branch on the first run, so no separate bootstrap is needed. After a second deploy, check that the footer build stamp on the live site changed.
