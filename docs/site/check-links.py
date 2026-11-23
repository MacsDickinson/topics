#!/usr/bin/env python3
"""Fail if a relative src/href/srcset/poster in any HTML file under a site directory
points outside that directory or at a missing file. Standard library only.

usage: python3 check-links.py <site-dir>
"""
import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlsplit

ATTRS = {"src", "href", "poster", "data", "action"}


class Refs(HTMLParser):
    def __init__(self):
        super().__init__()
        self.refs = []

    def handle_starttag(self, tag, attrs):
        for name, value in attrs:
            if value is None:
                continue
            if name in ATTRS:
                self.refs.append(value)
            elif name == "srcset":
                self.refs += [part.split()[0] for part in value.split(",") if part.strip()]

    handle_startendtag = handle_starttag


def main(root_arg):
    root = Path(root_arg).resolve()
    errors = []
    pages = sorted(root.rglob("*.html"))
    if not pages:
        errors.append(f"no HTML files under {root}")
    for page in pages:
        parser = Refs()
        parser.feed(page.read_text(encoding="utf-8"))
        for ref in parser.refs:
            ref = ref.strip()
            parts = urlsplit(ref)
            if not ref or ref.startswith("#") or parts.scheme or parts.netloc:
                continue  # in-page anchor, absolute URL (http, mailto, data...) or //host
            if ref.startswith("/"):
                errors.append(f"{page.relative_to(root)}: absolute path '{ref}' breaks Pages sub-paths")
                continue
            if not parts.path:
                continue
            target = (page.parent / unquote(parts.path)).resolve()
            try:
                target.relative_to(root)
            except ValueError:
                errors.append(f"{page.relative_to(root)}: '{ref}' leaves the site directory")
                continue
            if not target.exists():
                errors.append(f"{page.relative_to(root)}: '{ref}' not found")
    for e in errors:
        print("ERROR:", e)
    if not errors:
        print(f"Link check passed for {root_arg} ({len(pages)} page(s))")
    return 1 if errors else 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    sys.exit(main(sys.argv[1]))
