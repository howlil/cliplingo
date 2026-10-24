"""Build a reproducible importable ClipLingo EN-ID model pack (developer machine).

Install ctranslate2, transformers<5, torch, huggingface_hub; use a trusted pinned
model revision in production. This tool fetches model assets only when run by a
builder, never by the resident ClipLingo application.
"""
from __future__ import annotations
import argparse
import hashlib
from pathlib import Path
import shutil
import subprocess
from huggingface_hub import hf_hub_download

MODEL = "Helsinki-NLP/opus-mt-en-id"

def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("output", type=Path)
    ap.add_argument("--revision", required=True, help="Immutable Hugging Face commit SHA")
    args = ap.parse_args()
    folder = args.output.resolve()
    # CT2's converter requires the output directory to NOT EXIST, even if empty.
    if folder.exists():
        ap.error("output directory must not exist")
    folder.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run([
        "ct2-transformers-converter", "--model", MODEL,
        "--revision", args.revision, "--output_dir", str(folder),
        "--quantization", "int8_float32",
    ], check=True)
    for name in ("source.spm", "target.spm"):
        source = hf_hub_download(repo_id=MODEL, filename=name, revision=args.revision)
        shutil.copy2(source, folder / name)
    required = {"model.bin", "config.json", "source.spm", "target.spm"}
    present = {p.name for p in folder.iterdir() if p.is_file()}
    if not required <= present:
        raise RuntimeError(f"missing: {required - present}")
    lines = ["ClipLingoModelPackV1 en-id"]
    for path in sorted(folder.iterdir()):
        if not path.is_file() or not path.name.isascii():
            raise RuntimeError(f"invalid model file: {path.name}")
        with path.open("rb") as file:
            digest = hashlib.file_digest(file, "sha256").hexdigest()
        lines.append(f"{digest}  {path.name}")
    (folder / "cliplingo-model.sha256").write_text("\n".join(lines) + "\n", encoding="ascii")
    print(f"Built {folder} with {len(lines)-1} hashed model files")

if __name__ == "__main__":
    main()