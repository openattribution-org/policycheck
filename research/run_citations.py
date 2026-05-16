#!/usr/bin/env python3
"""Run citation collection across AI search providers.

Sends prompts to OpenAI, Gemini, and Perplexity, extracts cited URLs,
and writes results to CSV for compliance analysis.
"""

from __future__ import annotations

import argparse
import csv
import os
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
from pathlib import Path

from dotenv import load_dotenv

from providers import SearchProvider, SearchResult

load_dotenv()

# Provider imports are deferred to avoid import errors when keys are missing

PROVIDER_REGISTRY: dict[str, tuple[str, str]] = {
    "openai": ("providers.openai_provider", "OpenAIProvider"),
    "gemini": ("providers.gemini_provider", "GeminiProvider"),
    "perplexity": ("providers.perplexity_provider", "PerplexityProvider"),
    "anthropic": ("providers.anthropic_provider", "AnthropicProvider"),
}

ENV_KEYS: dict[str, str] = {
    "openai": "OPENAI_API_KEY",
    "gemini": "GEMINI_API_KEY",
    "perplexity": "PERPLEXITY_API_KEY",
    "anthropic": "ANTHROPIC_API_KEY",
}

CSV_COLUMNS = [
    "category",
    "prompt",
    "provider",
    "model",
    "run_number",
    "citation_rank",
    "citation_url",
    "domain",
    "citation_title",
    "timestamp",
]


def load_prompts(path: Path, limit: int | None = None) -> list[dict[str, str]]:
    """Read prompts from CSV file."""
    with open(path, newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        prompts = list(reader)
    if limit:
        prompts = prompts[:limit]
    return prompts


def detect_providers(requested: list[str] | None) -> list[str]:
    """Return provider names that have API keys set."""
    available = []
    for name, env_key in ENV_KEYS.items():
        if requested and name not in requested:
            continue
        if os.environ.get(env_key):
            available.append(name)
        elif requested and name in requested:
            print(f"  warning: {name} requested but {env_key} not set, skipping")
    return available


def init_provider(name: str) -> SearchProvider:
    """Dynamically import and instantiate a provider."""
    import importlib

    module_path, class_name = PROVIDER_REGISTRY[name]
    module = importlib.import_module(module_path)
    cls = getattr(module, class_name)
    return cls()


def search_with_retry(
    provider: SearchProvider, prompt: str, max_retries: int = 3
) -> SearchResult | None:
    """Call provider.search with exponential backoff on transient errors."""
    for attempt in range(max_retries):
        try:
            return provider.search(prompt)
        except KeyboardInterrupt:
            raise
        except Exception as e:
            wait = 2**attempt
            if attempt < max_retries - 1:
                time.sleep(wait)
            else:
                return None
    return None


def _write_citation_rows(
    writer: csv.DictWriter,
    category: str,
    prompt: str,
    name: str,
    model: str,
    run_num: int,
    result: SearchResult | None,
    now: str,
) -> None:
    """Write citation rows to CSV. Caller must hold the write lock."""
    if result is None:
        writer.writerow(
            {
                "category": category,
                "prompt": prompt,
                "provider": name,
                "model": model,
                "run_number": run_num,
                "citation_rank": "",
                "citation_url": "",
                "domain": "",
                "citation_title": "",
                "timestamp": now,
            }
        )
    elif result.citations:
        for cite in result.citations:
            writer.writerow(
                {
                    "category": category,
                    "prompt": prompt,
                    "provider": name,
                    "model": result.model,
                    "run_number": run_num,
                    "citation_rank": cite.rank,
                    "citation_url": cite.url,
                    "domain": cite.domain,
                    "citation_title": cite.title,
                    "timestamp": now,
                }
            )
    else:
        writer.writerow(
            {
                "category": category,
                "prompt": prompt,
                "provider": name,
                "model": result.model,
                "run_number": run_num,
                "citation_rank": 0,
                "citation_url": "",
                "domain": "",
                "citation_title": "",
                "timestamp": now,
            }
        )


def run(args: argparse.Namespace) -> None:
    """Main execution loop."""
    prompts_path = Path(__file__).parent / "prompts.csv"
    output_dir = Path(__file__).parent / "output"
    output_dir.mkdir(exist_ok=True)

    prompts = load_prompts(prompts_path, args.limit)

    if args.dry_run:
        provider_names = args.providers if args.providers else list(PROVIDER_REGISTRY)
        available_keys = [n for n in provider_names if os.environ.get(ENV_KEYS.get(n, ""))]
        missing_keys = [n for n in provider_names if n not in available_keys]

        total_calls = len(prompts) * len(provider_names) * args.runs
        print(f"Prompts:   {len(prompts)}")
        print(f"Providers: {', '.join(provider_names)}")
        if missing_keys:
            print(f"  missing API keys: {', '.join(missing_keys)}")
        print(f"Runs:      {args.runs}")
        print(f"Total:     {total_calls} API calls")
        print()

    provider_names = detect_providers(args.providers)

    if args.dry_run:
        print("Dry run — no API calls made.")
        print()
        for i, p in enumerate(prompts, 1):
            print(f"  {i:3d}. [{p['category']}] {p['prompt']}")
        return

    if not provider_names:
        print("error: no providers available (check API keys)")
        sys.exit(1)

    total_calls = len(prompts) * len(provider_names) * args.runs
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
    output_path = output_dir / f"raw_citations_{timestamp}.csv"

    print(f"Prompts:   {len(prompts)}")
    print(f"Providers: {', '.join(provider_names)}")
    print(f"Runs:      {args.runs}")
    print(f"Total:     {total_calls} API calls")
    print(f"Output:    {output_path}")
    print()

    # Initialise providers
    providers: dict[str, SearchProvider] = {}
    for name in provider_names:
        try:
            providers[name] = init_provider(name)
            print(f"  {name}: initialised ({providers[name].model})")
        except Exception as e:
            print(f"  {name}: failed to initialise — {e}")

    if not providers:
        print("error: no providers could be initialised")
        sys.exit(1)

    print()

    # Open CSV for incremental writes
    csv_file = open(output_path, "w", newline="", encoding="utf-8")
    writer = csv.DictWriter(csv_file, fieldnames=CSV_COLUMNS)
    writer.writeheader()

    write_lock = threading.Lock()
    print_lock = threading.Lock()
    call_counter = [0]  # mutable counter for threads
    total_citations = 0
    provider_stats: dict[str, dict[str, int]] = {
        name: {"calls": 0, "citations": 0, "failures": 0} for name in providers
    }

    def do_search(
        name: str,
        provider: SearchProvider,
        prompt: str,
        run_num: int,
    ) -> tuple[str, str, int, SearchResult | None]:
        """Run a single search call. Returns (name, model, run_num, result)."""
        result = search_with_retry(provider, prompt)
        return (name, provider.model, run_num, result)

    try:
        for prompt_idx, prompt_data in enumerate(prompts):
            category = prompt_data["category"]
            prompt = prompt_data["prompt"]
            short_prompt = prompt[:50] + "..." if len(prompt) > 50 else prompt

            # Build all (provider, run) tasks for this prompt
            tasks = [
                (name, provider, run_num)
                for name, provider in providers.items()
                for run_num in range(1, args.runs + 1)
            ]

            # Run all providers x runs concurrently for this prompt
            with ThreadPoolExecutor(max_workers=len(providers) * args.runs) as pool:
                futures = {
                    pool.submit(do_search, name, provider, prompt, run_num): (
                        name,
                        run_num,
                    )
                    for name, provider, run_num in tasks
                }

                for future in as_completed(futures):
                    name, model, run_num, result = future.result()
                    now = datetime.now(timezone.utc).isoformat()

                    with print_lock:
                        call_counter[0] += 1
                        n = call_counter[0]
                        if result is None:
                            provider_stats[name]["failures"] += 1
                            print(
                                f"[{n}/{total_calls}] {name} | "
                                f"Run {run_num}/{args.runs} | "
                                f"{category}: \"{short_prompt}\" | FAILED"
                            )
                        else:
                            n_cites = len(result.citations)
                            provider_stats[name]["calls"] += 1
                            provider_stats[name]["citations"] += n_cites
                            print(
                                f"[{n}/{total_calls}] {name} | "
                                f"Run {run_num}/{args.runs} | "
                                f"{category}: \"{short_prompt}\" | "
                                f"{n_cites} citations"
                            )

                    with write_lock:
                        _write_citation_rows(
                            writer,
                            category,
                            prompt,
                            name,
                            model,
                            run_num,
                            result,
                            now,
                        )
                        csv_file.flush()

            # Delay between prompts, not between every single call
            if args.delay > 0 and prompt_idx < len(prompts) - 1:
                time.sleep(args.delay)

    except KeyboardInterrupt:
        print("\n\nInterrupted — partial results saved.")
    finally:
        csv_file.close()

    total_citations = sum(s["citations"] for s in provider_stats.values())

    # Summary
    print()
    print("=" * 60)
    print("Summary")
    print("=" * 60)
    print(f"Output: {output_path}")
    print(f"Total citations collected: {total_citations}")
    print()
    for name, stats in provider_stats.items():
        print(
            f"  {name}: {stats['calls']} successful calls, "
            f"{stats['citations']} citations, "
            f"{stats['failures']} failures"
        )


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Collect citations from AI search providers"
    )
    parser.add_argument(
        "--providers",
        type=lambda s: s.split(","),
        default=None,
        help="Comma-separated provider list (default: all with API keys)",
    )
    parser.add_argument(
        "--runs",
        type=int,
        default=3,
        help="Number of runs per prompt per provider (default: 3)",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Limit to first N prompts (default: all)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print plan without making API calls",
    )
    parser.add_argument(
        "--delay",
        type=float,
        default=1.0,
        help="Seconds between prompts (default: 1.0)",
    )
    args = parser.parse_args()
    run(args)


if __name__ == "__main__":
    main()
