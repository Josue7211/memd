#!/usr/bin/env python3
import argparse
import contextlib
import importlib.util
import json
import subprocess
import sys
import tempfile
import uuid
from pathlib import Path

import chromadb


REPO_ROOT = Path(__file__).resolve().parents[1]
OUTPUT_ROOT = REPO_ROOT / ".memd" / "benchmarks" / "baselines"
REPLAYS_PATH = OUTPUT_ROOT / "mempalace_replays.json"
ARTIFACTS_ROOT = OUTPUT_ROOT / "mempalace-replays"
MEMPALACE_ROOT = Path(
    (
        Path(__file__).resolve().parents[2] / "mempalace"
        if len(Path(__file__).resolve().parents) > 2
        else REPO_ROOT.parent / "mempalace"
    )
).resolve()
MEMPALACE_PYTHON = MEMPALACE_ROOT / ".venv" / "bin" / "python"


def ensure_environment() -> None:
    if not MEMPALACE_ROOT.exists():
        raise SystemExit(f"missing mempalace repo: {MEMPALACE_ROOT}")
    if not MEMPALACE_PYTHON.exists():
        raise SystemExit(f"missing mempalace venv python: {MEMPALACE_PYTHON}")
    OUTPUT_ROOT.mkdir(parents=True, exist_ok=True)
    ARTIFACTS_ROOT.mkdir(parents=True, exist_ok=True)


def load_module(name: str, module_path: Path):
    spec = importlib.util.spec_from_file_location(name, module_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load module: {module_path}")
    module = importlib.util.module_from_spec(spec)
    sys.path.insert(0, str(MEMPALACE_ROOT))
    try:
        spec.loader.exec_module(module)
    finally:
        sys.path.pop(0)
    return module


def run_subprocess(command: list[str], stdout_path: Path) -> None:
    with stdout_path.open("w", encoding="utf-8") as log:
        subprocess.run(
            command,
            cwd=MEMPALACE_ROOT,
            check=True,
            stdout=log,
            stderr=subprocess.STDOUT,
            text=True,
        )


def artifact_dir(benchmark_id: str) -> Path:
    path = ARTIFACTS_ROOT / benchmark_id / "latest"
    path.mkdir(parents=True, exist_ok=True)
    return path


def average(values) -> float:
    values = list(values)
    if not values:
        return 0.0
    return sum(values) / len(values)


def write_json(path: Path, payload) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def longmemeval_dataset_path() -> Path:
    return REPO_ROOT / ".memd" / "benchmarks" / "datasets" / "longmemeval" / "longmemeval_s_cleaned.json"


def locomo_dataset_path() -> Path:
    return REPO_ROOT / ".memd" / "benchmarks" / "datasets" / "locomo" / "locomo10.json"


def convomem_dataset_path() -> Path:
    return REPO_ROOT / ".memd" / "benchmarks" / "datasets" / "convomem" / "convomem-evidence-sample.json"


def membench_dataset_path() -> Path:
    return REPO_ROOT / ".memd" / "benchmarks" / "datasets" / "membench" / "membench-firstagent.json"


def run_longmemeval() -> dict:
    bench = "longmemeval"
    out_dir = artifact_dir(bench)
    results_path = out_dir / "results.jsonl"
    log_path = out_dir / "stdout.log"
    summary_path = out_dir / "summary.json"
    command = [
        str(MEMPALACE_PYTHON),
        "benchmarks/longmemeval_bench.py",
        str(longmemeval_dataset_path()),
        "--mode",
        "raw",
        "--out",
        str(results_path),
    ]
    run_subprocess(command, log_path)
    with results_path.open(encoding="utf-8") as handle:
        rows = [json.loads(line) for line in handle if line.strip()]
    accuracy = average(
        row["retrieval_results"]["metrics"]["session"]["recall_any@5"] for row in rows
    )
    summary = {
        "accuracy": accuracy,
        "artifact_path": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/",
        "command": f"{MEMPALACE_PYTHON} {REPO_ROOT / 'scripts' / 'bench-mempalace.py'} --benchmark {bench}",
        "note": "local same-fixture replay complete; MemPalace raw session recall_any@5 on memd cached LongMemEval fixture",
        "source": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/summary.json",
        "status": "replayed",
    }
    write_json(summary_path, summary)
    return summary


def run_locomo() -> dict:
    bench = "locomo"
    out_dir = artifact_dir(bench)
    results_path = out_dir / "results.json"
    log_path = out_dir / "stdout.log"
    summary_path = out_dir / "summary.json"
    command = [
        str(MEMPALACE_PYTHON),
        "benchmarks/locomo_bench.py",
        str(locomo_dataset_path()),
        "--mode",
        "hybrid",
        "--granularity",
        "session",
        "--top-k",
        "10",
        "--out",
        str(results_path),
    ]
    run_subprocess(command, log_path)
    rows = json.loads(results_path.read_text(encoding="utf-8"))
    accuracy = average(row["recall"] for row in rows)
    summary = {
        "accuracy": accuracy,
        "artifact_path": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/",
        "command": f"{MEMPALACE_PYTHON} {REPO_ROOT / 'scripts' / 'bench-mempalace.py'} --benchmark {bench}",
        "note": "local same-fixture replay complete; MemPalace hybrid session top-10 average retrieval recall on memd cached LoCoMo fixture",
        "source": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/summary.json",
        "status": "replayed",
    }
    write_json(summary_path, summary)
    return summary


def run_convomem() -> dict:
    bench = "convomem"
    out_dir = artifact_dir(bench)
    results_path = out_dir / "results.json"
    log_path = out_dir / "stdout.log"
    summary_path = out_dir / "summary.json"
    fixture = json.loads(convomem_dataset_path().read_text(encoding="utf-8"))
    results = []
    with log_path.open("w", encoding="utf-8") as log:
        with contextlib.redirect_stdout(log), contextlib.redirect_stderr(log):
            print(f"ConvoMem exact-fixture replay: {len(fixture['items'])} items")
            for index, item in enumerate(fixture["items"], start=1):
                recall, details = retrieve_convomem_item(
                    question=item["query"],
                    conversations=item["metadata"]["conversations"],
                    message_evidences=item["metadata"]["message_evidences"],
                    top_k=10,
                )
                results.append(
                    {
                        "item_id": item["item_id"],
                        "question": item["query"],
                        "answer": item["gold_answer"],
                        "recall": recall,
                        "details": details,
                    }
                )
                if index % 25 == 0 or index == len(fixture["items"]):
                    print(f"[{index}/{len(fixture['items'])}] avg_recall={average(r['recall'] for r in results):.3f}")
    write_json(results_path, results)
    accuracy = average(row["recall"] for row in results)
    summary = {
        "accuracy": accuracy,
        "artifact_path": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/",
        "command": f"{MEMPALACE_PYTHON} {REPO_ROOT / 'scripts' / 'bench-mempalace.py'} --benchmark {bench}",
        "note": "local same-fixture replay complete; MemPalace raw top-10 replay over memd normalized ConvoMem evidence fixture",
        "source": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/summary.json",
        "status": "replayed",
    }
    write_json(summary_path, summary)
    return summary


def run_membench() -> dict:
    bench = "membench"
    out_dir = artifact_dir(bench)
    results_path = out_dir / "results.json"
    log_path = out_dir / "stdout.log"
    summary_path = out_dir / "summary.json"
    module = load_module("mempalace_membench_bench", MEMPALACE_ROOT / "benchmarks" / "membench_bench.py")
    raw_fixture = json.loads(membench_dataset_path().read_text(encoding="utf-8"))
    with tempfile.TemporaryDirectory(prefix="memd-mempalace-membench-") as temp_dir:
        temp_dir_path = Path(temp_dir)
        temp_file = temp_dir_path / "simple.json"
        write_json(temp_file, raw_fixture)
        with log_path.open("w", encoding="utf-8") as log:
            with contextlib.redirect_stdout(log), contextlib.redirect_stderr(log):
                results = module.run_membench(
                    str(temp_dir_path),
                    categories=["simple"],
                    topic="",
                    top_k=5,
                    limit=0,
                    mode="hybrid",
                    out_file=str(results_path),
                )
    if results is None:
        raise RuntimeError("MemPalace MemBench replay produced no results")
    accuracy = average(1.0 if row["hit_at_k"] else 0.0 for row in results)
    summary = {
        "accuracy": accuracy,
        "artifact_path": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/",
        "command": f"{MEMPALACE_PYTHON} {REPO_ROOT / 'scripts' / 'bench-mempalace.py'} --benchmark {bench}",
        "note": "local same-fixture replay complete; MemPalace hybrid top-5 over memd 3000-item MemBench combined fixture via synthetic single-category adapter",
        "source": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/summary.json",
        "status": "replayed",
    }
    write_json(summary_path, summary)
    return summary


RUNNERS = {
    "longmemeval": run_longmemeval,
    "locomo": run_locomo,
    "convomem": run_convomem,
    "membench": run_membench,
}


def load_existing_replays() -> dict:
    if REPLAYS_PATH.exists():
        return json.loads(REPLAYS_PATH.read_text(encoding="utf-8"))
    discovered = {}
    for summary_path in ARTIFACTS_ROOT.glob("*/latest/summary.json"):
        try:
            discovered[summary_path.parent.parent.name] = json.loads(
                summary_path.read_text(encoding="utf-8")
            )
        except json.JSONDecodeError:
            continue
    return discovered


def retrieve_convomem_item(
    question: str,
    conversations: list[dict],
    message_evidences: list[dict],
    top_k: int,
) -> tuple[float, dict]:
    corpus = []
    speakers = []
    for conversation in conversations:
        for message in conversation.get("messages", []):
            corpus.append(message["text"])
            speakers.append(message["speaker"])
    if not corpus:
        return 0.0, {"error": "empty corpus"}
    evidence_texts = {
        evidence["text"].strip().lower()
        for evidence in message_evidences
        if evidence.get("text")
    }
    client = chromadb.EphemeralClient()
    try:
        collection = client.create_collection(f"mempal_drawers_{uuid.uuid4().hex}")
        collection.add(
            documents=corpus,
            ids=[f"msg_{index}" for index in range(len(corpus))],
            metadatas=[{"speaker": speaker, "idx": index} for index, speaker in enumerate(speakers)],
        )
        result = collection.query(
            query_texts=[question],
            n_results=min(top_k, len(corpus)),
            include=["documents", "metadatas"],
        )
        retrieved_indices = [metadata["idx"] for metadata in result["metadatas"][0]]
        retrieved_texts = [corpus[index].strip().lower() for index in retrieved_indices]
        found = 0
        for evidence_text in evidence_texts:
            if any(
                evidence_text in retrieved_text or retrieved_text in evidence_text
                for retrieved_text in retrieved_texts
            ):
                found += 1
        recall = found / len(evidence_texts) if evidence_texts else 1.0
        return recall, {
            "retrieved_count": len(retrieved_indices),
            "evidence_count": len(evidence_texts),
            "found": found,
        }
    finally:
        client = None


def main() -> int:
    parser = argparse.ArgumentParser(description="Run local MemPalace replays for memd public benchmarks.")
    parser.add_argument(
        "--benchmark",
        action="append",
        choices=sorted(RUNNERS.keys()),
        help="Benchmark to run. Repeat to select multiple. Default: all.",
    )
    args = parser.parse_args()
    ensure_environment()
    selected = args.benchmark or list(RUNNERS.keys())
    replays = load_existing_replays()
    for benchmark_id in selected:
        print(f"[mempalace] replaying {benchmark_id}")
        replays[benchmark_id] = RUNNERS[benchmark_id]()
        write_json(REPLAYS_PATH, replays)
        print(f"[mempalace] {benchmark_id} accuracy={replays[benchmark_id]['accuracy']:.3f}")
    print(f"[mempalace] wrote {REPLAYS_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
