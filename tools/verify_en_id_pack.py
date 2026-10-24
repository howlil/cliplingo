"""Validate the exact local EN->ID pack using the real CTranslate2 Python runtime.

This is a release/CI proof, NOT an alternative runtime for ClipLingo.exe.
Mirrors the native source SentencePiece -> EOS -> translate -> target decode path.
"""
from __future__ import annotations

import argparse
from pathlib import Path
import re

import ctranslate2
import sentencepiece as spm


def verify(pack: Path) -> None:
    manifest = pack / "cliplingo-model.sha256"
    lines = manifest.read_text(encoding="ascii").splitlines()
    if not lines or lines[0] != "ClipLingoModelPackV1 en-id":
        raise RuntimeError("unexpected model pack format")
    import hashlib

    included: set[str] = set()
    for row in lines[1:]:
        match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z0-9_.-]+)", row)
        if match is None:
            raise RuntimeError(f"invalid manifest row: {row!r}")
        digest, name = match.groups()
        if name in included or hashlib.sha256((pack / name).read_bytes()).hexdigest() != digest:
            raise RuntimeError(f"missing, duplicate or modified model file: {name}")
        included.add(name)
    if included != {p.name for p in pack.iterdir() if p.is_file() and p.name != manifest.name}:
        raise RuntimeError("pack includes files outside the manifest")

    source = spm.SentencePieceProcessor(model_file=str(pack / "source.spm"))
    target = spm.SentencePieceProcessor(model_file=str(pack / "target.spm"))
    translator = ctranslate2.Translator(str(pack), device="cpu", inter_threads=1, intra_threads=2)
    for english in ("Hello world!", "The book is on the table.", "Thank you for your help."):
        source_pieces = source.encode(english, out_type=str) + ["</s>"]
        outputs = translator.translate_batch([source_pieces], beam_size=1)
        if len(outputs) != 1 or not outputs[0].hypotheses:
            raise RuntimeError(f"missing translation for {english!r}")
        pieces = [p for p in outputs[0].hypotheses[0] if p not in ("</s>", "<pad>")]
        translated = target.decode(pieces).strip()
        if not translated or translated == english or "<unk>" in translated:
            raise RuntimeError(f"invalid translation: {english!r} -> {translated!r}")
        print(f"EN {english} -> ID {translated}")
    print(f"Real CTranslate2 + SentencePiece pack validated (CT2 {ctranslate2.__version__})")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("pack", type=Path)
    verify(parser.parse_args().pack)
