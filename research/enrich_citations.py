#!/usr/bin/env python3
"""Enrich raw citations CSV with robots.txt compliance data.

Reads a raw_citations CSV produced by run_citations.py, queries the
policycheck server for each unique domain, and appends bot_blocked
and robots_status columns based on provider→bot mapping.
"""

from __future__ import annotations

import argparse
import csv
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import urlparse

import requests

# Per provider, the two bot identities the audit checks.
# "training" is the crawler the provider uses for model training; this is the
# bot most publishers list explicitly when they want to opt out of AI use.
# "live" is the user agent the provider's web_search / grounding tool uses
# when fetching pages at answer-time. Publishers' opt-out signals usually
# do not name these live agents, so the two rates can diverge sharply.
PROVIDER_BOT_MAP: dict[str, dict[str, str]] = {
    "openai":     {"training": "GPTBot",          "live": "OAI-SearchBot"},
    "gemini":     {"training": "Google-Extended", "live": "Google-Extended"},
    "perplexity": {"training": "PerplexityBot",   "live": "Perplexity-User"},
    "anthropic":  {"training": "ClaudeBot",       "live": "Claude-SearchBot"},
}


def extract_domains(input_path: Path) -> list[str]:
    """Return unique non-empty domains from the CSV."""
    domains: set[str] = set()
    with open(input_path, newline="", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            domain = row.get("domain", "").strip()
            if domain:
                domains.add(domain)
    return sorted(domains)


def fetch_batch(
    server: str, domains: list[str], timeout: float = 60.0
) -> dict:
    """POST a batch of domains to the policycheck /analyze endpoint.

    Returns the JSON response or raises on error.
    """
    urls = [f"https://{d}" for d in domains]
    resp = requests.post(
        f"{server}/analyze",
        json={"urls": urls},
        timeout=timeout,
    )
    resp.raise_for_status()
    return resp.json()


def build_lookup(
    server: str, domains: list[str], batch_size: int
) -> dict[str, dict]:
    """Analyse all domains in batches and build a lookup dict.

    Returns: {domain: {"bots": {bot_name: "blocked"|"allowed"}, "status": str}}
    """
    lookup: dict[str, dict] = {}
    total = len(domains)

    for i in range(0, total, batch_size):
        batch = domains[i : i + batch_size]
        batch_num = i // batch_size + 1
        total_batches = (total + batch_size - 1) // batch_size
        print(
            f"  Batch {batch_num}/{total_batches} "
            f"({len(batch)} domains)...",
            end="",
            flush=True,
        )

        try:
            data = fetch_batch(server, batch)
        except requests.RequestException as e:
            print(f" ERROR: {e}")
            # Mark all domains in this batch as fetch errors
            for domain in batch:
                lookup[domain] = {"bots": {}, "status": "fetcherror"}
            continue

        for result in data.get("results", []):
            # Extract domain from the URL we sent
            parsed = urlparse(result.get("url", ""))
            domain = parsed.hostname or ""

            status = result.get("status", "fetcherror")
            bots: dict[str, str] = {}
            for bot in result.get("ai_bot_analysis", []):
                bots[bot["bot_name"]] = bot["status"]

            lookup[domain] = {"bots": bots, "status": status}

        print(f" done")

        # Small delay between batches to be polite
        if i + batch_size < total:
            time.sleep(0.5)

    return lookup


def enrich(
    input_path: Path,
    output_path: Path,
    lookup: dict[str, dict],
) -> dict[str, dict[str, int]]:
    """Read input CSV, append compliance columns, write output CSV.

    Returns per-provider stats: {provider: {"total": n, "blocked": n}}.
    """
    stats: dict[str, dict[str, int]] = {}

    with open(input_path, newline="", encoding="utf-8") as fin:
        reader = csv.DictReader(fin)
        in_fields = list(reader.fieldnames or [])
        new_fields = [
            "training_bot",
            "training_bot_blocked",
            "live_bot",
            "live_bot_blocked",
            "bot_blocked",  # alias for training_bot_blocked; preserved for back-compat
            "robots_status",
        ]
        out_fields = in_fields + [f for f in new_fields if f not in in_fields]

        with open(output_path, "w", newline="", encoding="utf-8") as fout:
            writer = csv.DictWriter(fout, fieldnames=out_fields)
            writer.writeheader()

            for row in reader:
                domain = row.get("domain", "").strip()
                provider = row.get("provider", "").strip()
                bot_pair = PROVIDER_BOT_MAP.get(provider, {})
                training_bot = bot_pair.get("training", "")
                live_bot = bot_pair.get("live", "")

                info = lookup.get(domain, {})
                robots_status = info.get("status", "")
                bots = info.get("bots", {})

                def status_for(name: str) -> str:
                    if not domain or not name:
                        return ""
                    if name not in bots:
                        return ""
                    return str(bots[name] == "blocked").lower()

                training_blocked = status_for(training_bot)
                live_blocked = status_for(live_bot)

                row["training_bot"] = training_bot
                row["training_bot_blocked"] = training_blocked
                row["live_bot"] = live_bot
                row["live_bot_blocked"] = live_blocked
                row["bot_blocked"] = training_blocked  # back-compat alias
                row["robots_status"] = robots_status
                writer.writerow(row)

                # Track stats for summary
                if provider and training_blocked:
                    s = stats.setdefault(provider, {
                        "training_total": 0, "training_blocked": 0,
                        "live_total": 0,     "live_blocked": 0,
                    })
                    s["training_total"] += 1
                    if training_blocked == "true":
                        s["training_blocked"] += 1
                if provider and live_blocked:
                    s = stats.setdefault(provider, {
                        "training_total": 0, "training_blocked": 0,
                        "live_total": 0,     "live_blocked": 0,
                    })
                    s["live_total"] += 1
                    if live_blocked == "true":
                        s["live_blocked"] += 1

    return stats


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Enrich citations CSV with robots.txt compliance data"
    )
    parser.add_argument(
        "input_csv",
        type=Path,
        help="Path to raw_citations CSV",
    )
    parser.add_argument(
        "--server",
        default="http://localhost:3000",
        help="PolicyCheck server URL (default: http://localhost:3000)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Output path (default: output/enriched_citations_{timestamp}.csv)",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=50,
        help="URLs per /analyze request (default: 50, max 100)",
    )
    args = parser.parse_args()

    if not args.input_csv.exists():
        print(f"error: {args.input_csv} not found")
        sys.exit(1)

    if args.batch_size < 1 or args.batch_size > 100:
        print("error: batch-size must be between 1 and 100")
        sys.exit(1)

    output_dir = Path(__file__).parent / "output"
    output_dir.mkdir(exist_ok=True)

    if args.output:
        output_path = args.output
    else:
        timestamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
        output_path = output_dir / f"enriched_citations_{timestamp}.csv"

    # Step 1: Extract unique domains
    print(f"Reading {args.input_csv}...")
    domains = extract_domains(args.input_csv)
    print(f"  {len(domains)} unique domains found")
    print()

    # Step 2: Check server health
    try:
        health = requests.get(f"{args.server}/health", timeout=5)
        health.raise_for_status()
        print(f"Server: {args.server} (healthy)")
    except requests.RequestException as e:
        print(f"error: cannot reach server at {args.server} — {e}")
        print("Start the server first: cargo run -p policycheck -- serve --port 3000")
        sys.exit(1)

    # Step 3: Analyse domains in batches
    print(f"Analysing {len(domains)} domains (batch size: {args.batch_size})...")
    lookup = build_lookup(args.server, domains, args.batch_size)
    print()

    # Step 4: Enrich and write output
    print(f"Writing {output_path}...")
    stats = enrich(args.input_csv, output_path, lookup)
    print()

    # Summary
    print("=" * 50)
    print("Summary")
    print("=" * 50)
    print(f"Input:   {args.input_csv}")
    print(f"Output:  {output_path}")
    print(f"Domains: {len(domains)} unique, {len(lookup)} analysed")
    print()

    for provider, s in sorted(stats.items()):
        pair = PROVIDER_BOT_MAP.get(provider, {})
        training_bot = pair.get("training", "?")
        live_bot = pair.get("live", "?")
        t_rate = (
            s["training_blocked"] / s["training_total"] * 100
            if s["training_total"] > 0 else 0
        )
        l_rate = (
            s["live_blocked"] / s["live_total"] * 100
            if s["live_total"] > 0 else 0
        )
        print(f"  {provider}:")
        print(
            f"    training [{training_bot:<18}] "
            f"{s['training_blocked']}/{s['training_total']} blocked ({t_rate:.1f}%)"
        )
        print(
            f"    live     [{live_bot:<18}] "
            f"{s['live_blocked']}/{s['live_total']} blocked ({l_rate:.1f}%)"
        )


if __name__ == "__main__":
    main()
