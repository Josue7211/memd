#!/usr/bin/env python3
import argparse
import contextlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import uuid
from pathlib import Path


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
BENCH_LIMIT = 0


def ensure_mempalace_python_path() -> None:
    site_packages = sorted((MEMPALACE_ROOT / ".venv" / "lib").glob("python*/site-packages"))
    for path in reversed(site_packages):
        path_text = str(path)
        if path_text not in sys.path:
            sys.path.insert(0, path_text)


def ensure_environment() -> None:
    if not MEMPALACE_ROOT.exists():
        raise SystemExit(f"missing mempalace repo: {MEMPALACE_ROOT}")
    if not MEMPALACE_PYTHON.exists():
        raise SystemExit(f"missing mempalace venv python: {MEMPALACE_PYTHON}")
    ensure_mempalace_python_path()
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


def replay_command(benchmark_id: str) -> str:
    command = f"{MEMPALACE_PYTHON} {REPO_ROOT / 'scripts' / 'bench-mempalace.py'} --benchmark {benchmark_id}"
    if BENCH_LIMIT > 0:
        command = f"{command} --limit {BENCH_LIMIT}"
    return command


def external_public_cache_root() -> Path:
    explicit = os.environ.get("DATASET_CACHE_DIR") or os.environ.get(
        "MEMD_EXTERNAL_PUBLIC_CACHE_DIR"
    )
    if explicit:
        return Path(explicit).expanduser()
    return Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache")) / "memd" / "external-public-cache"


def dataset_path(benchmark_id: str, filename: str) -> Path:
    primary = REPO_ROOT / ".memd" / "benchmarks" / "datasets" / benchmark_id / filename
    if primary.exists():
        return primary
    external_cache = (
        external_public_cache_root()
        / benchmark_id
        / "benchmarks"
        / "datasets"
        / benchmark_id
        / filename
    )
    if external_cache.exists():
        return external_cache
    return primary


def longmemeval_dataset_path() -> Path:
    return dataset_path("longmemeval", "longmemeval_s_cleaned.json")


def locomo_dataset_path() -> Path:
    return dataset_path("locomo", "locomo10.json")


def convomem_dataset_path() -> Path:
    return dataset_path("convomem", "convomem-evidence-sample.json")


def membench_dataset_path() -> Path:
    return dataset_path("membench", "membench-firstagent.json")


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
    if BENCH_LIMIT > 0:
        command.extend(["--limit", str(BENCH_LIMIT)])
    run_subprocess(command, log_path)
    with results_path.open(encoding="utf-8") as handle:
        rows = [json.loads(line) for line in handle if line.strip()]
    accuracy = average(
        row["retrieval_results"]["metrics"]["session"]["recall_any@5"] for row in rows
    )
    summary = {
        "accuracy": accuracy,
        "artifact_path": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/",
        "command": replay_command(bench),
        "limit": BENCH_LIMIT or None,
        "limit_scope": "items",
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
    data_file = locomo_dataset_path()
    temp_dir = None
    item_limit = None
    limit_scope = None
    if BENCH_LIMIT > 0:
        temp_dir = tempfile.TemporaryDirectory(prefix="memd-mempalace-locomo-")
        data_file = Path(temp_dir.name) / "locomo-item-slice.json"
        item_limit = write_locomo_item_slice(locomo_dataset_path(), data_file, BENCH_LIMIT)
        limit_scope = "items"
    command = [
        str(MEMPALACE_PYTHON),
        "benchmarks/locomo_bench.py",
        str(data_file),
        "--mode",
        "hybrid",
        "--granularity",
        "session",
        "--top-k",
        "10",
        "--out",
        str(results_path),
    ]
    try:
        run_subprocess(command, log_path)
        rows = json.loads(results_path.read_text(encoding="utf-8"))
    finally:
        if temp_dir is not None:
            temp_dir.cleanup()
    accuracy = average(row["recall"] for row in rows)
    summary = {
        "accuracy": accuracy,
        "artifact_path": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/",
        "command": replay_command(bench),
        "limit": item_limit,
        "limit_scope": limit_scope,
        "note": "local same-fixture replay complete; MemPalace hybrid session top-10 average retrieval recall on memd cached LoCoMo fixture item slice",
        "source": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/summary.json",
        "status": "replayed",
    }
    write_json(summary_path, summary)
    return summary


def write_locomo_item_slice(source_path: Path, target_path: Path, limit: int) -> int:
    source_rows = json.loads(source_path.read_text(encoding="utf-8"))
    remaining = limit
    sliced_rows = []
    for row in source_rows:
        if remaining <= 0:
            break
        qa_rows = list(row.get("qa") or [])
        if not qa_rows:
            continue
        kept = qa_rows[:remaining]
        if not kept:
            continue
        row_copy = dict(row)
        row_copy["qa"] = kept
        sliced_rows.append(row_copy)
        remaining -= len(kept)
    write_json(target_path, sliced_rows)
    return limit - remaining


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
            items = fixture["items"][:BENCH_LIMIT] if BENCH_LIMIT > 0 else fixture["items"]
            for index, item in enumerate(items, start=1):
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
                if index % 25 == 0 or index == len(items):
                    print(f"[{index}/{len(items)}] avg_recall={average(r['recall'] for r in results):.3f}")
    write_json(results_path, results)
    accuracy = average(row["recall"] for row in results)
    summary = {
        "accuracy": accuracy,
        "artifact_path": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/",
        "command": replay_command(bench),
        "limit": BENCH_LIMIT or None,
        "limit_scope": "items",
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
                    limit=BENCH_LIMIT,
                    mode="hybrid",
                    out_file=str(results_path),
                )
    if results is None:
        raise RuntimeError("MemPalace MemBench replay produced no results")
    accuracy = average(1.0 if row["hit_at_k"] else 0.0 for row in results)
    summary = {
        "accuracy": accuracy,
        "artifact_path": f".memd/benchmarks/baselines/mempalace-replays/{bench}/latest/",
        "command": replay_command(bench),
        "limit": BENCH_LIMIT or None,
        "limit_scope": "items",
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
    import chromadb

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
    global BENCH_LIMIT
    parser = argparse.ArgumentParser(description="Run local MemPalace replays for memd public benchmarks.")
    parser.add_argument(
        "--benchmark",
        action="append",
        choices=sorted(RUNNERS.keys()),
        help="Benchmark to run. Repeat to select multiple. Default: all.",
    )
    parser.add_argument("--limit", type=int, default=0, help="Limit replay items where benchmark adapters support item-level limits.")
    args = parser.parse_args()
    BENCH_LIMIT = max(args.limit, 0)
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
