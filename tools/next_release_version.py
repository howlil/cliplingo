#!/usr/bin/env python3
"""Select the next ClipLingo beta tag from existing Git tags.

No GitHub API mutation and no reservation/tag creation: publishing creates the
tag only after the full portable package passes acceptance.
"""
from __future__ import annotations

import re
import sys

PREFIX = "v0.1.0-beta."
PATTERN = re.compile(r"^v0\.1\.0-beta\.([1-9][0-9]*)$")


def next_version(tags: list[str]) -> str:
    numbers = [
        int(match.group(1))
        for tag in tags
        if (match := PATTERN.fullmatch(tag.strip())) is not None
    ]
    return f"{PREFIX}{max(numbers, default=0) + 1}"


if __name__ == "__main__":
    print(next_version(sys.stdin.read().splitlines()))
