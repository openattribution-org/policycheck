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

# Which crawler bot each provider uses for training/indexing
PROVIDER_BOT_MAP: dict[str, str] = {
    "openai": "GPTBot",
    "gemini": "Google-Extended",
    "perplexity": "PerplexityBot",
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
        out_fields = in_fields + ["bot_blocked", "robots_status"]

        with open(output_path, "w", newline="", encoding="utf-8") as fout:
            writer = csv.DictWriter(fout, fieldnames=out_fields)
            writer.writeheader()

            for row in reader:
                domain = row.get("domain", "").strip()
                provider = row.get("provider", "").strip()
                bot_name = PROVIDER_BOT_MAP.get(provider)

                info = lookup.get(domain, {})
                robots_status = info.get("status", "")
                bots = info.get("bots", {})

                if not domain or not bot_name:
                    bot_blocked = ""
                elif bot_name in bots:
                    bot_blocked = str(bots[bot_name] == "blocked").lower()
                else:
                    bot_blocked = ""

                row["bot_blocked"] = bot_blocked
                row["robots_status"] = robots_status
                writer.writerow(row)

                # Track stats for summary
                if provider and bot_blocked:
                    if provider not in stats:
                        stats[provider] = {"total": 0, "blocked": 0}
                    stats[provider]["total"] += 1
                    if bot_blocked == "true":
                        stats[provider]["blocked"] += 1

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
        bot = PROVIDER_BOT_MAP.get(provider, "?")
        rate = s["blocked"] / s["total"] * 100 if s["total"] > 0 else 0
        print(
            f"  {provider} ({bot}): "
            f"{s['blocked']}/{s['total']} citations blocked "
            f"({rate:.1f}%)"
        )


if __name__ == "__main__":
    main()
