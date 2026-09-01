#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import sys
from pathlib import Path

REQUIRED_SCHEMA = "cliplingo.model-pack/v1"
REQUIRED_STAGE_FILES = ("config.json", "model.bin", "source.spm", "target.spm")


def load_catalog(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as handle:
        catalog = json.load(handle)

    if catalog.get("schema") != REQUIRED_SCHEMA:
        raise ValueError(f"unsupported catalog schema: {catalog.get('schema')!r}")
    if catalog.get("route") != ["ja", "en", "id"]:
        raise ValueError("alpha catalog route must be ja -> en -> id")
    if catalog.get("runtime", {}).get("engine") != "ctranslate2":
        raise ValueError("alpha catalog runtime must be ctranslate2")
    if catalog.get("runtime", {}).get("quantization") != "int8":
        raise ValueError("alpha catalog quantization must be int8")

    stages = catalog.get("stages")
    if not isinstance(stages, list) or len(stages) != 2:
        raise ValueError("alpha catalog must contain exactly two ordered stages")

    expected_pairs = [("ja", "en"), ("en", "id")]
    for index, (stage, expected_pair) in enumerate(zip(stages, expected_pairs, strict=True)):
        actual_pair = (stage.get("source_language"), stage.get("target_language"))
        if actual_pair != expected_pair:
            raise ValueError(
                f"stage {index} must be {expected_pair[0]} -> {expected_pair[1]}, got {actual_pair}"
            )
        revision = stage.get("revision", "")
        if len(revision) != 40 or any(ch not in "0123456789abcdef" for ch in revision):
            raise ValueError(f"stage {stage.get('id')} must pin a 40-character git revision")
        if stage.get("license") != "Apache-2.0":
            raise ValueError(f"stage {stage.get('id')} must use the distributable Apache-2.0 baseline")
        copy_files = set(stage.get("copy_files", []))
        if not {"source.spm", "target.spm"}.issubset(copy_files):
            raise ValueError(f"stage {stage.get('id')} must preserve SentencePiece models")

    return catalog


def print_plan(catalog: dict, output: Path) -> None:
    print(f"pack={catalog['id']}")
    print(f"route={' -> '.join(catalog['route'])}")
    print(f"engine={catalog['runtime']['engine']}@{catalog['runtime']['version']}")
    print(f"quantization={catalog['runtime']['quantization']}")
    for stage in catalog["stages"]:
        print(
            "stage="
            f"{stage['id']} model={stage['model']} revision={stage['revision']} "
            f"output={output / stage['output_directory']}"
        )


def build_pack(catalog: dict, output: Path) -> None:
    try:
        from ctranslate2.converters import TransformersConverter
    except ImportError as error:
        raise RuntimeError(
            "model build dependencies are missing; install scripts/models/requirements.txt"
        ) from error

    if output.exists():
        shutil.rmtree(output)
    output.mkdir(parents=True)

    for stage in catalog["stages"]:
        stage_output = output / stage["output_directory"]
        converter = TransformersConverter(
            stage["model"],
            revision=stage["revision"],
            copy_files=stage["copy_files"],
        )
        converter.convert(
            str(stage_output),
            quantization=catalog["runtime"]["quantization"],
            force=False,
        )

        missing = [name for name in REQUIRED_STAGE_FILES if not (stage_output / name).is_file()]
        if missing:
            raise RuntimeError(f"stage {stage['id']} is incomplete; missing {', '.join(missing)}")

    manifest = {
        "schema": catalog["schema"],
        "id": catalog["id"],
        "source_language": catalog["source_language"],
        "target_language": catalog["target_language"],
        "route": catalog["route"],
        "runtime": catalog["runtime"],
        "stages": [
            {
                "id": stage["id"],
                "source_language": stage["source_language"],
                "target_language": stage["target_language"],
                "model": stage["model"],
                "revision": stage["revision"],
                "license": stage["license"],
                "directory": stage["output_directory"],
            }
            for stage in catalog["stages"]
        ],
    }
    with (output / "manifest.json").open("w", encoding="utf-8", newline="\n") as handle:
        json.dump(manifest, handle, indent=2, ensure_ascii=False)
        handle.write("\n")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Build a ClipLingo OPUS-MT language pack")
    parser.add_argument("--catalog", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="validate the catalog and print the pinned build plan without downloading model weights",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        catalog = load_catalog(args.catalog)
        print_plan(catalog, args.output)
        if not args.dry_run:
            build_pack(catalog, args.output)
    except (OSError, ValueError, RuntimeError) as error:
        print(f"model-pack error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
