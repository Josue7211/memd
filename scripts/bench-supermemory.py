#!/usr/bin/env python3
import argparse
import json
import os
import time
import urllib.error
import urllib.request
import uuid
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
OUTPUT_ROOT = REPO_ROOT / ".memd" / "benchmarks" / "baselines"
REPLAYS_PATH = OUTPUT_ROOT / "supermemory_replays.json"
ARTIFACTS_ROOT = OUTPUT_ROOT / "supermemory-replays"
API_BASE = os.environ.get("SUPERMEMORY_API_BASE", "https://api.supermemory.ai").rstrip("/")
API_KEY = os.environ.get("SUPERMEMORY_API_KEY", "")
RUN_ID = os.environ.get("SUPERMEMORY_RUN_ID", uuid.uuid4().hex[:12])


def write_json(path: Path, payload) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def dataset_path(benchmark_id: str, filename: str) -> Path:
    primary = REPO_ROOT / ".memd" / "benchmarks" / "datasets" / benchmark_id / filename
    if primary.exists():
        return primary
    explicit = os.environ.get("DATASET_CACHE_DIR") or os.environ.get(
        "MEMD_EXTERNAL_PUBLIC_CACHE_DIR"
    )
    cache_root = (
        Path(explicit).expanduser()
        if explicit
        else Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache"))
        / "memd"
        / "external-public-cache"
    )
    cached = (
        cache_root
        / benchmark_id
        / "benchmarks"
        / "datasets"
        / benchmark_id
        / filename
    )
    return cached if cached.exists() else primary


def require_api_key() -> None:
    if not API_KEY:
        raise SystemExit("missing SUPERMEMORY_API_KEY")


def http_json(method: str, path: str, payload: dict, timeout: int = 60) -> dict:
    data = json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        f"{API_BASE}{path}",
        data=data,
        method=method,
        headers={
            "Authorization": f"Bearer {API_KEY}",
            "Content-Type": "application/json",
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            raw = response.read().decode("utf-8")
            return json.loads(raw) if raw else {}
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"supermemory {method} {path} failed {exc.code}: {body}") from exc


def chunks(values, size):
    for index in range(0, len(values), size):
        yield values[index : index + size]


def create_memories(container_tag: str, docs: list[dict]) -> int:
    created = 0
    for batch in chunks(docs, 100):
        memories = [
            {
                "content": f"[source_id: {doc['id']}]\n{doc['text'][:9500]}",
                "isStatic": True,
                "metadata": {
                    "source_id": doc["id"],
                    "benchmark": doc.get("benchmark", ""),
                    "question_id": doc.get("question_id", ""),
                },
            }
            for doc in batch
        ]
        http_json(
            "POST",
            "/v4/memories",
            {"containerTag": container_tag, "memories": memories},
            timeout=120,
        )
        created += len(memories)
    return created


def search_memories(container_tag: str, query: str, top_k: int) -> list[dict]:
    response = http_json(
        "POST",
        "/v4/search",
        {
            "q": query,
            "containerTag": container_tag,
            "searchMode": "hybrid",
            "limit": top_k,
            "threshold": 0.0,
            "rerank": True,
        },
        timeout=120,
    )
    return response.get("results") or []


def result_source_id(result: dict) -> str:
    metadata = result.get("metadata") or {}
    if isinstance(metadata, dict) and metadata.get("source_id"):
        return str(metadata["source_id"])
    text = result.get("memory") or result.get("chunk") or ""
    marker = "[source_id:"
    if marker in text:
        return text.split(marker, 1)[1].split("]", 1)[0].strip()
    return str(result.get("id") or "")


def render_messages(messages: list[dict]) -> str:
    parts = []
    for message in messages:
        role = message.get("role") or message.get("speaker") or "unknown"
        text = message.get("content") or message.get("text") or ""
        if text:
            parts.append(f"{role}: {text}")
    return "\n".join(parts)


def longmemeval_cases(limit: int) -> list[dict]:
    rows = json.loads(dataset_path("longmemeval", "longmemeval_s_cleaned.json").read_text())
    cases = []
    for row in rows[:limit]:
        docs = []
        for sid, date, session in zip(
            row.get("haystack_session_ids") or [],
            row.get("haystack_dates") or [],
            row.get("haystack_sessions") or [],
        ):
            docs.append(
                {
                    "id": sid,
                    "text": f"({date})\n{render_messages(session)}",
                    "benchmark": "longmemeval",
                    "question_id": row.get("question_id", ""),
                }
            )
        cases.append(
            {
                "question_id": row.get("question_id", ""),
                "query": row.get("question", ""),
                "expected": set(map(str, row.get("answer_session_ids") or [])),
                "docs": docs,
            }
        )
    return cases


def locomo_cases(limit: int) -> list[dict]:
    rows = json.loads(dataset_path("locomo", "locomo10.json").read_text())
    cases = []
    for row in rows:
        conversation = row.get("conversation") or {}
        docs = []
        for key, dialogs in conversation.items():
            if not key.startswith("session_") or not isinstance(dialogs, list):
                continue
            session_index = key.split("_", 1)[1]
            date = conversation.get(f"session_{session_index}_date_time", "")
            for dialog in dialogs:
                dia_id = str(dialog.get("dia_id") or "")
                text = dialog.get("text") or ""
                speaker = dialog.get("speaker") or "unknown"
                if dia_id and text:
                    docs.append(
                        {
                            "id": dia_id,
                            "text": f"({date}) {speaker}: {text}",
                            "benchmark": "locomo",
                            "question_id": row.get("sample_id", ""),
                        }
                    )
        for index, qa in enumerate(row.get("qa") or []):
            if len(cases) >= limit:
                return cases
            evidence = set()
            for target in qa.get("evidence") or []:
                for part in str(target).replace(",", ";").split(";"):
                    part = part.strip()
                    if part:
                        evidence.add(part)
            cases.append(
                {
                    "question_id": f"{row.get('sample_id', 'locomo')}::{index}",
                    "query": qa.get("question", ""),
                    "expected": evidence,
                    "docs": docs,
                }
            )
    return cases


def render_membench_turn(turn: dict) -> str:
    user = turn.get("user")
    assistant = turn.get("assistant") or turn.get("response") or turn.get("agent")
    if user and assistant:
        return f"user: {user}\nassistant: {assistant}"
    if user:
        return f"user: {user}"
    if assistant:
        return f"assistant: {assistant}"
    return ""


def membench_cases(limit: int) -> list[dict]:
    raw = json.loads(dataset_path("membench", "membench-firstagent.json").read_text())
    cases = []
    for topic in sorted(raw.keys()):
        for item_index, item in enumerate(raw.get(topic) or []):
            if len(cases) >= limit:
                return cases
            docs = []
            for session_index, session in enumerate(item.get("message_list") or []):
                for turn in session:
                    text = render_membench_turn(turn)
                    mid = turn.get("mid", 0)
                    doc_id = json.dumps([mid, session_index], separators=(",", ":"))
                    docs.append(
                        {
                            "id": doc_id,
                            "text": text,
                            "benchmark": "membench",
                            "question_id": f"{topic}::{item_index}",
                        }
                    )
            qa = item.get("QA") or {}
            expected = {
                json.dumps(target, separators=(",", ":"))
                for target in (qa.get("target_step_id") or [])
            }
            cases.append(
                {
                    "question_id": f"{topic}::{item_index}",
                    "query": qa.get("question", ""),
                    "expected": expected,
                    "docs": docs,
                }
            )
    return cases


def convomem_cases(limit: int) -> list[dict]:
    fixture = json.loads(dataset_path("convomem", "convomem-evidence-sample.json").read_text())
    cases = []
    for item in (fixture.get("items") or [])[:limit]:
        docs = []
        expected = set()
        evidence_texts = {
            (evidence.get("text") or "").strip().lower()
            for evidence in item.get("metadata", {}).get("message_evidences", [])
        }
        for conversation in item.get("metadata", {}).get("conversations", []):
            for index, message in enumerate(conversation.get("messages", [])):
                text = message.get("text") or ""
                doc_id = f"{conversation.get('id', 'conv')}::msg:{index}"
                docs.append(
                    {
                        "id": doc_id,
                        "text": f"{message.get('speaker', 'unknown')}: {text}",
                        "benchmark": "convomem",
                        "question_id": item.get("question_id", ""),
                    }
                )
                text_lower = text.strip().lower()
                if any(
                    evidence_text and (evidence_text in text_lower or text_lower in evidence_text)
                    for evidence_text in evidence_texts
                ):
                    expected.add(doc_id)
        cases.append(
            {
                "question_id": item.get("question_id", ""),
                "query": item.get("query", ""),
                "expected": expected,
                "docs": docs,
            }
        )
    return cases


CASE_BUILDERS = {
    "longmemeval": longmemeval_cases,
    "locomo": locomo_cases,
    "membench": membench_cases,
    "convomem": convomem_cases,
}


def run_benchmark(benchmark: str, limit: int, top_k: int, run_label: str) -> dict:
    cases = CASE_BUILDERS[benchmark](limit)
    out_dir = ARTIFACTS_ROOT / benchmark / "latest"
    out_dir.mkdir(parents=True, exist_ok=True)
    rows = []
    hits = 0
    started = time.time()
    for index, case in enumerate(cases, start=1):
        container = f"memd-25-5-{run_label}-{benchmark}-{index}"
        created = create_memories(container, case["docs"])
        results = search_memories(container, case["query"], top_k)
        retrieved = [result_source_id(result) for result in results]
        expected = set(case["expected"])
        hit = bool(expected and expected.intersection(retrieved[:top_k]))
        hits += int(hit)
        rows.append(
            {
                "question_id": case["question_id"],
                "query": case["query"],
                "expected": sorted(expected),
                "retrieved": retrieved[:top_k],
                "hit": hit,
                "created": created,
            }
        )
    accuracy = hits / len(cases) if cases else 0.0
    result = {
        "status": "replayed",
        "accuracy": accuracy,
        "limit": len(cases),
        "limit_scope": "items",
        "top_k": top_k,
        "api_base": API_BASE,
        "artifact_path": f".memd/benchmarks/baselines/supermemory-replays/{benchmark}/latest/",
        "command": f"SUPERMEMORY_API_KEY=... {REPO_ROOT / 'scripts' / 'bench-supermemory.py'} --benchmark {benchmark} --limit {limit}",
        "source": f".memd/benchmarks/baselines/supermemory-replays/{benchmark}/latest/summary.json",
        "elapsed_seconds": round(time.time() - started, 3),
    }
    write_json(out_dir / "results.json", rows)
    write_json(out_dir / "summary.json", result)
    return result


def load_existing_replays() -> dict:
    if REPLAYS_PATH.exists():
        return json.loads(REPLAYS_PATH.read_text(encoding="utf-8"))
    return {}


def main() -> int:
    parser = argparse.ArgumentParser(description="Run live Supermemory same-fixture replays.")
    parser.add_argument("--benchmark", action="append", choices=sorted(CASE_BUILDERS.keys()))
    parser.add_argument("--limit", type=int, default=50)
    parser.add_argument("--top-k", type=int, default=5)
    parser.add_argument("--run-label", default=RUN_ID)
    args = parser.parse_args()
    require_api_key()
    OUTPUT_ROOT.mkdir(parents=True, exist_ok=True)
    replays = load_existing_replays()
    for benchmark in args.benchmark or CASE_BUILDERS.keys():
        print(f"[supermemory] replaying {benchmark}")
        replays[benchmark] = run_benchmark(benchmark, args.limit, args.top_k, args.run_label)
        write_json(REPLAYS_PATH, replays)
        print(f"[supermemory] {benchmark} accuracy={replays[benchmark]['accuracy']:.3f}")
    print(f"[supermemory] wrote {REPLAYS_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
