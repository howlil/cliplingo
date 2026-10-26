#!/usr/bin/env python3
"""F2.7 headless Windows model comparison. No user content; public fixtures only.

Runs the real compiled offline_model_smoke binary in fresh processes.
Collects sampled per-process CPU seconds, Windows working set/private commit,
one-reference corpus chrF (diagnostic only), and per-fixture output parity.
No changes to production beam or thread defaults.
"""
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
import re
import subprocess
import time

import psutil
from sacrebleu.metrics import CHRF

MAX_SECONDS = 240
POLL_SECONDS = 0.10


def fixture_rows(path: Path) -> list[dict]:
    rows = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(rows, list) or not (10 <= len(rows) <= 100):
        raise ValueError("Expected 10..100 public fixtures")
    ids = set()
    for row in rows:
        if not isinstance(row, dict) or not all(
            isinstance(row.get(field), str) and row[field].strip()
            for field in ("id", "category", "source", "reference")
        ):
            raise ValueError("Invalid quality fixture")
        if row["id"] in ids:
            raise ValueError("Duplicate fixture identifier")
        ids.add(row["id"])
    return rows


def quality_result(path: Path, expected_ids: list[str]) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    rows = data.get("results", [])
    if data.get("schema") != 1 or len(rows) != len(expected_ids):
        raise ValueError("Wrong native quality output schema/count")
    if [row.get("id") for row in rows] != expected_ids:
        raise ValueError("Native inference changed fixture ordering or IDs")
    if not all(isinstance(row.get("target"), str) and row["target"].strip()
               and "\0" not in row["target"] for row in rows):
        raise ValueError("Model emitted blank/invalid output")
    return data


def sample_process_tree(root: psutil.Process) -> tuple[int, int, float]:
    try:
        processes = [root, *root.children(recursive=True)]
    except (psutil.NoSuchProcess, psutil.AccessDenied):
        processes = [root]
    rss, private, cpu = 0, 0, 0.0
    for child in processes:
        try:
            with child.oneshot():
                mem = child.memory_info()
                rss += mem.rss  # Windows working set, contains shared pages.
                private += getattr(mem, "private", mem.vms)  # Windows private commit.
                times = child.cpu_times()
                cpu += times.user + times.system
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
    return rss, private, cpu


def run_profile(binary: Path, fixtures: Path, output: Path, beam: int, threads: int) -> dict:
    command = [
        str(binary), "--beam", str(beam), "--threads", str(threads),
        "--quality-suite", str(fixtures), "--quality-output", str(output),
    ]
    started = time.monotonic()
    proc = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                            text=True, encoding="utf-8", errors="replace")
    root = psutil.Process(proc.pid)
    samples = []
    try:
        while proc.poll() is None:
            if time.monotonic() - started > MAX_SECONDS:
                proc.kill()
                raise TimeoutError(f"Native model profile beam={beam} threads={threads} timed out")
            samples.append(sample_process_tree(root))
            time.sleep(POLL_SECONDS)
        # Best-effort final sample: may race the process exit.
        stdout, _ = proc.communicate(timeout=10)
        if proc.returncode != 0:
            raise RuntimeError(f"Native model failed (exit {proc.returncode}): {stdout[-1500:]}")
        if not output.is_file():
            raise RuntimeError("Native model returned success without writing quality report")
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.communicate()
    elapsed = time.monotonic() - started
    if not samples:
        raise RuntimeError("No process samples collected")
    peak_rss = max(s[0] for s in samples)
    peak_private = max(s[1] for s in samples)
    cpu_seconds = max(s[2] for s in samples)  # approx; final tail may be missed.
    return {
        "beam_size": beam, "cpu_threads": threads,
        "wall_seconds": round(elapsed, 3),
        "cpu_seconds_sampled": round(cpu_seconds, 3),
        "cpu_core_equivalents_approx": round(cpu_seconds / max(0.001, elapsed), 3),
        "peak_sampled_working_set_mib": round(peak_rss / (1024 * 1024), 1),
        "peak_sampled_private_commit_mib": round(peak_private / (1024 * 1024), 1),
        "sampling_interval_ms": int(POLL_SECONDS * 1000),
        "sample_count": len(samples),
        "quality_output": str(output),
    }


def compare_quality(fixtures: list[dict], baseline: dict, greedy: dict) -> dict:
    target_a = [r["target"] for r in baseline["results"]]
    target_b = [r["target"] for r in greedy["results"]]
    refs = [r["reference"] for r in fixtures]
    scorer = CHRF(word_order=0, char_order=6, beta=2)
    score_a = round(scorer.corpus_score(target_a, [refs]).score, 2)
    score_b = round(scorer.corpus_score(target_b, [refs]).score, 2)
    changed = [row["id"] for row, a, b in zip(fixtures, target_a, target_b) if a != b]

    by_category = {}
    for category in sorted({r["category"] for r in fixtures}):
        indices = [i for i, r in enumerate(fixtures) if r["category"] == category]
        by_category[category] = {
            "n": len(indices),
            "baseline_chrF": round(scorer.corpus_score(
                [target_a[i] for i in indices], [[refs[i] for i in indices]]
            ).score, 2),
            "greedy_chrF": round(scorer.corpus_score(
                [target_b[i] for i in indices], [[refs[i] for i in indices]]
            ).score, 2),
        }

    def digit_sequence(text: str) -> str:
        return "".join(re.findall(r"\d", text))

    digit_mismatch = {}
    for label, targets in (("baseline", target_a), ("greedy", target_b)):
        digit_mismatch[label] = [
            row["id"] for row, translation in zip(fixtures, targets)
            if row["category"] == "numbers"
            and digit_sequence(translation) != digit_sequence(row["source"])
        ]
    return {
        "metric": "SacreBLEU CHRF(6,0,beta=2); single human-authored reference per item",
        "baseline_chrF": score_a,
        "greedy_chrF": score_b,
        "target_output_changes": len(changed),
        "changed_fixture_ids": changed,
        "category_diagnostics": by_category,
        "digit_preservation_diagnostic_ids": digit_mismatch,
        "interpretation": "Exploratory, not proof of fluent/faithful translations; "
                          "one reference per item and a small authored suite.",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--fixtures", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    binary, fixtures = args.binary.resolve(), args.fixtures.resolve()
    if not binary.is_file():
        parser.error(f"compiled native binary not found: {binary}")
    rows = fixture_rows(fixtures)
    out = args.output_dir.resolve()
    out.mkdir(parents=True, exist_ok=True)

    # Identical model on same CI host; fresh process per profile. Fixed order
    # remains a confounder; repeat counterbalanced runs before product decisions.
    results = []
    for beam, threads, name in [(2, 2, "baseline"), (1, 2, "greedy")]:
        filepath = out / f"{name}-public-results.json"
        stats = run_profile(binary, fixtures, filepath, beam, threads)
        verified = quality_result(filepath, [r["id"] for r in rows])
        results.append((name, stats, verified))
    metrics = compare_quality(rows, results[0][2], results[1][2])
    report = {
        "schema": 1,
        "scope": "Headless native Windows EN-ID model; NOT UIA to DWM E2E",
        "fixture_count": len(rows),
        "profiles": {name: values for name, values, _ in results},
        "quality": metrics,
        "decision": "Do not change product default without repeated runs, "
                    "human review and interactive Windows CPU/RAM/popup evidence.",
    }
    report_path = out / "comparison-summary.json"
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n",
                           encoding="utf-8")
    # Public fixture aggregate only; no model output/user content printed.
    print("cliplingo_f27_summary=" + json.dumps({
        "fixture_count": len(rows),
        "baseline_chrF": metrics["baseline_chrF"],
        "greedy_chrF": metrics["greedy_chrF"],
        "changed_outputs": metrics["target_output_changes"],
        "profiles": {
            name: {
                "wall_seconds": values["wall_seconds"],
                "cpu_seconds_sampled": values["cpu_seconds_sampled"],
                "peak_working_mib": values["peak_sampled_working_set_mib"],
                "peak_private_mib": values["peak_sampled_private_commit_mib"],
            } for name, values, _ in results
        },
    }, ensure_ascii=False))


if __name__ == "__main__":
    main()
