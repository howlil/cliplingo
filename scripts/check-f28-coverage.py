#!/usr/bin/env python3
"""Fixed-public-example semantic coverage guard for Issue #28.

This is a narrow corpus regression check, NOT a general semantic-equivalence
metric. The genuine CTranslate2 Windows model must emit the target content.
"""
import argparse
import json
from pathlib import Path


def validate(path: Path) -> None:
    document = json.loads(path.read_text(encoding="utf-8"))
    assert document["beam_size"] == 2 and document["cpu_threads"] == 2, (
        "Coverage gate must use unchanged production default"
    )
    rows = document["results"]
    by_id = {row["id"]: row["target"].casefold() for row in rows}
    if len(by_id) != len(rows):
        raise AssertionError("Duplicate or malformed quality fixture IDs")
    # The original monolithic baseline dropped the opening report sentence.
    # Distinct concept anchors verify both independent sentences survived.
    anchors = {
        "paragraph": ["laporan", "pengaturan", "waktu", "hasil"],
        "multisentence": ["teks", "kursor"],
    }
    for fixture_id, concepts in anchors.items():
        target = by_id[fixture_id]
        missing = [concept for concept in concepts if concept not in target]
        if missing:
            raise AssertionError(
                f"Missing semantic content for {fixture_id}: {missing}; "
                "inspect downloaded CI artifact for the public model output"
            )
    print("cliplingo_f28_semantic_guard=PASS fixed_public_cases=2; "
          "human review and adversarial paragraph tests remain required")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline-output", type=Path, required=True)
    args = parser.parse_args()
    validate(args.baseline_output)


if __name__ == "__main__":
    main()
