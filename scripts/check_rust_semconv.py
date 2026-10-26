#!/usr/bin/env python3
"""Verify that the Rust semconv crate covers every attribute key declared
in the canonical Python semconv source.

Fails fast (exit 1) if any string value declared as
`ATTR_CONSTANT = "value"` in
`python/openinference-semantic-conventions/src/openinference/semconv/trace/__init__.py`
does not appear in the Rust semconv crate, either as the right-hand side
of a `pub const NAME: &str = "value";` item or as an
`Enum::Variant => "value"` match arm.

We compare on the *values* (the wire-format keys), not the constant
names, because Rust and Python use different naming conventions
(UpperCamelCase enum variants vs ALL_CAPS) for the same values. The
wire-format value is the thing the collector cares about and the thing
that must not drift across SDKs.

Run from the repo root:

    python3 scripts/check_rust_semconv.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
PYTHON_SOURCE = (
    REPO_ROOT
    / "python"
    / "openinference-semantic-conventions"
    / "src"
    / "openinference"
    / "semconv"
    / "trace"
    / "__init__.py"
)
RUST_SOURCES = [
    REPO_ROOT / "rust" / "openinference-semantic-conventions" / "src" / "attributes.rs",
    REPO_ROOT / "rust" / "openinference-semantic-conventions" / "src" / "enums.rs",
]

# Python value patterns we extract:
#   FOO = "bar.baz"
#   FOO: SomeType = "bar.baz"
# We deliberately accept only top-level / class-level string assignments
# (no f-strings, no concatenations) — anything more dynamic isn't a
# semconv "constant" worth pinning.
PY_ASSIGN = re.compile(r'^\s+([A-Z][A-Z0-9_]*)\s*(?::[^=]+)?=\s*"([^"]+)"')

# Rust patterns, matched against the whole file because rustfmt wraps
# long items onto a second line:
#   pub const FOO: &str = "bar.baz";
#   pub const FOO: &str =
#       "bar.baz";
#   SpanKind::Llm => "LLM",
# Test tables use `(SpanKind::Llm, "LLM")` tuples, not `=>`, so they
# don't count toward coverage.
RUST_CONST = re.compile(
    r"^\s*pub\s+const\s+([A-Z][A-Z0-9_]*)\s*:\s*&(?:'static\s+)?str\s*=\s*\"([^\"]+)\"\s*;",
    re.MULTILINE,
)
RUST_MATCH_ARM = re.compile(
    r'^\s*([A-Z][A-Za-z0-9]*::[A-Z][A-Za-z0-9]*)\s*=>\s*"([^"]+)"',
    re.MULTILINE,
)


def read_source(path: Path) -> str:
    if not path.is_file():
        sys.exit(f"check_rust_semconv: missing source file {path}")
    return path.read_text()


def extract_python_values(path: Path) -> set[str]:
    # Line by line, exactly as scripts/check_go_semconv.py does, so both
    # checks pin the same set of Python values.
    out: set[str] = set()
    for line in read_source(path).splitlines():
        m = PY_ASSIGN.search(line)
        if m:
            out.add(m.group(2))
    return out


def extract_rust_values(path: Path) -> set[str]:
    text = read_source(path)
    out: set[str] = set()
    for pattern in (RUST_CONST, RUST_MATCH_ARM):
        for m in pattern.finditer(text):
            out.add(m.group(2))
    return out


def main() -> int:
    py_values = extract_python_values(PYTHON_SOURCE)
    rust_values: set[str] = set()
    for rust_src in RUST_SOURCES:
        rust_values |= extract_rust_values(rust_src)

    missing = py_values - rust_values
    if missing:
        print(
            "Rust semconv is missing the following wire-format keys "
            "that the Python source declares:",
            file=sys.stderr,
        )
        for v in sorted(missing):
            print(f"  - {v}", file=sys.stderr)
        print(
            "\nAdd the constants in rust/openinference-semantic-conventions/src/attributes.rs "
            "or .../enums.rs, then re-run this script.",
            file=sys.stderr,
        )
        return 1

    # Inverse direction is informational: Rust may declare values that
    # don't yet exist in Python (e.g., during a parity gap). We report
    # them but do NOT fail.
    extras = rust_values - py_values
    if extras:
        print(
            f"info: {len(extras)} Rust value(s) not present in Python "
            "(probably intentional — Rust-side additions land first):"
        )
        for v in sorted(extras):
            print(f"  + {v}")

    print(
        f"OK — {len(py_values)} Python wire-format keys all present "
        f"in Rust semconv ({len(rust_values)} total Rust values)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
