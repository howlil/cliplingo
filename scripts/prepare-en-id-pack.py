#!/usr/bin/env python3
"""Developer-only conversion into a verified ClipLingo EN->ID model pack.

Requirements (NOT needed by ClipLingo at runtime):
  python -m pip install "ctranslate2==4.8.3" "transformers>=4.49,<5" sentencepiece
    huggingface_hub torch
  python scripts/prepare-en-id-pack.py --output "<LOCALAPPDATA>/ClipLingo/models/en-id"

The output is created only if absent. No source text is uploaded by ClipLingo.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

MODEL = "Helsinki-NLP/opus-mt-en-id"
# Explicit upstream revision to keep tokenizer/weights reproducible.
REVISION = "052fd659487dd0d1a655969050e97950f83dc903"
UPSTREAM_FILES = [
    "config.json", "generation_config.json", "pytorch_model.bin",
    "source.spm", "target.spm", "tokenizer_config.json", "vocab.json",
]
ACCEPT = {
    "model.bin", "config.json", "source.spm", "target.spm",
    "shared_vocabulary.json", "source_vocabulary.json", "target_vocabulary.json",
}
REQUIRED = {"model.bin", "config.json", "source.spm", "target.spm"}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    dest = args.output.expanduser().resolve()
    if dest.exists():
        parser.error(f"Target already exists; refusing to overwrite: {dest}")
    dest.parent.mkdir(parents=True, exist_ok=True)
    from huggingface_hub import snapshot_download
    with tempfile.TemporaryDirectory(prefix="cliplingo-model-") as tmp:
        root = Path(tmp)
        # Download only checkpoint + tokenizer: never fetch the duplicate ~280 MB
        # TensorFlow checkpoint. Xet transfers are chunked/resumable when installed.
        original = None
        for attempt in range(3):
            try:
                original = Path(snapshot_download(
                    repo_id=MODEL, revision=REVISION, local_dir=root / "upstream",
                    allow_patterns=UPSTREAM_FILES, max_workers=2,
                ))
                break
            except Exception:
                if attempt == 2:
                    raise
                time.sleep(4 * (attempt + 1))
        assert original is not None
        converted = root / "converted"
        converter = shutil.which("ct2-transformers-converter")
        if converter is None:
            raise RuntimeError("Install ctranslate2 package first (ct2-transformers-converter)")
        subprocess.run([
            converter, "--model", str(original), "--output_dir", str(converted),
            "--quantization", "int8",
        ], check=True)
        for name in ("source.spm", "target.spm"):
            source = original / name
            if not source.is_file():
                raise RuntimeError(f"Missing required tokenizer: {source}")
            shutil.copyfile(source, converted / name)
        available = {p.name for p in converted.iterdir() if p.is_file()}
        if not REQUIRED.issubset(available):
            raise RuntimeError(f"Converted pack missing: {sorted(REQUIRED - available)}")
        if not available.issubset(ACCEPT):
            raise RuntimeError(f"Unexpected conversion files: {sorted(available - ACCEPT)}")
        checksums = {}
        for path in converted.iterdir():
            if path.is_file():
                digest = hashlib.sha256()
                with path.open("rb") as stream:
                    for block in iter(lambda: stream.read(65536), b""):
                        digest.update(block)
                checksums[path.name] = digest.hexdigest()
        manifest = {
            "schema": 1,
            "model_id": MODEL,
            "language_pair": "en-id",
            "format": "ctranslate2-int8",
            "files": dict(sorted(checksums.items())),
        }
        (converted / "manifest.json").write_text(
            json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
        )
        # Staging and atomic first-install rename on the same volume.
        staging = dest.parent / f".{dest.name}.staging-{os.getpid()}"
        if staging.exists():
            raise RuntimeError(f"Staging path exists: {staging}")
        try:
            shutil.copytree(converted, staging)
            staging.rename(dest)
        finally:
            if staging.exists():
                shutil.rmtree(staging)
    print(f"Verified-pack input ready at: {dest}")
    print("Final SHA-256 checks are repeated by the Rust app on first model load.")
    print("Benchmark/quality validation on an actual Windows host is still required.")


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        print(f"Pack preparation failed: {exc}", file=sys.stderr)
        sys.exit(1)
