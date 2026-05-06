#!/usr/bin/env python3
"""Minimal third-party replay fixture for V13 release evidence."""

import json
import sys


def main() -> int:
    payload = json.load(open(sys.argv[1], encoding="utf-8"))
    if payload.get("schema") != "memd.release-0.1.0.v1":
        return 2
    return 0 if payload.get("replay_expected") == "all_answers_match" else 1


if __name__ == "__main__":
    raise SystemExit(main())
