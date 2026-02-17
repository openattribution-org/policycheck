#!/usr/bin/env python3
"""Generate a self-contained HTML report from enriched citations CSV.

Reads the output of enrich_citations.py and produces a single HTML file
with charts (inline SVG via matplotlib) and tables, styled to match
the OpenAttribution website design language.
"""

from __future__ import annotations

import argparse
import csv
import io
import base64
import sys
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker

# OpenAttribution brand colours
CORAL_600 = "#dc3b35"
CORAL_500 = "#f5564e"
CORAL_100 = "#fee2e1"
CORAL_50 = "#fef2f2"
AMBER_600 = "#d97706"
AMBER_500 = "#f59e0b"
AMBER_100 = "#fef3c7"
AMBER_50 = "#fffbeb"
GRAY_700 = "#374151"
GRAY_600 = "#4b5563"
GRAY_400 = "#9ca3af"
GRAY_200 = "#e5e7eb"
CREAM = "#faf8f5"

PROVIDER_BOT_MAP = {
    "openai": "GPTBot",
    "gemini": "Google-Extended",
    "perplexity": "PerplexityBot",
}

PROVIDER_COLOURS = {
    "openai": CORAL_600,
    "gemini": AMBER_600,
    "perplexity": "#7c3aed",
}

PROVIDER_DISPLAY_NAMES = {
    "openai": "OpenAI",
    "gemini": "Gemini",
    "perplexity": "Perplexity",
}


def load_csv(path: Path) -> list[dict]:
    with open(path, newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def fig_to_base64(fig: plt.Figure) -> str:
    """Render a matplotlib figure to a base64-encoded PNG."""
    buf = io.BytesIO()
    fig.savefig(buf, format="png", dpi=150, bbox_inches="tight",
                facecolor="white", edgecolor="none")
    plt.close(fig)
    buf.seek(0)
    return base64.b64encode(buf.read()).decode("ascii")


def setup_plot_style():
    """Set matplotlib defaults to match OA design."""
    plt.rcParams.update({
        "font.family": "sans-serif",
        "font.sans-serif": ["Inter", "Helvetica Neue", "Arial", "sans-serif"],
        "font.size": 11,
        "font.weight": "light",
        "axes.titleweight": "normal",
        "axes.titlesize": 14,
        "axes.labelsize": 11,
        "axes.labelweight": "light",
        "axes.spines.top": False,
        "axes.spines.right": False,
        "axes.edgecolor": GRAY_200,
        "axes.facecolor": "white",
        "figure.facecolor": "white",
        "xtick.color": GRAY_600,
        "ytick.color": GRAY_600,
        "grid.color": GRAY_200,
        "grid.linewidth": 0.5,
    })


def compute_stats(rows: list[dict]) -> dict:
    """Compute all metrics from enriched CSV rows."""
    # Filter to rows with actual citations (non-empty domain)
    citations = [r for r in rows if r.get("domain", "").strip()]

    total_citations = len(citations)
    unique_domains = set(r["domain"] for r in citations)
    unique_prompts = set(r["prompt"] for r in citations)
    providers_seen = set(r["provider"] for r in citations)
    categories_seen = set(r["category"] for r in citations)

    # Per-provider violation rates
    provider_stats = {}
    for provider in sorted(providers_seen):
        prows = [r for r in citations if r["provider"] == provider]
        has_data = [r for r in prows if r.get("bot_blocked", "").strip()]
        blocked = [r for r in has_data if r["bot_blocked"] == "true"]
        provider_stats[provider] = {
            "total": len(prows),
            "checked": len(has_data),
            "blocked": len(blocked),
            "rate": len(blocked) / len(has_data) * 100 if has_data else 0,
        }

    # Per-category violation rates
    category_stats = {}
    for cat in sorted(categories_seen):
        cat_rows = [r for r in citations if r["category"] == cat]
        has_data = [r for r in cat_rows if r.get("bot_blocked", "").strip()]
        blocked = [r for r in has_data if r["bot_blocked"] == "true"]
        cat_domains = set(r["domain"] for r in cat_rows)
        category_stats[cat] = {
            "citations": len(cat_rows),
            "domains": len(cat_domains),
            "checked": len(has_data),
            "blocked": len(blocked),
            "rate": len(blocked) / len(has_data) * 100 if has_data else 0,
        }

    # Top cited domains with compliance status
    domain_counts: Counter = Counter()
    domain_blocked: dict[str, set] = defaultdict(set)
    domain_providers: dict[str, set] = defaultdict(set)
    for r in citations:
        d = r["domain"]
        domain_counts[d] += 1
        domain_providers[d].add(r["provider"])
        if r.get("bot_blocked") == "true":
            domain_blocked[d].add(r["provider"])

    top_domains = []
    for domain, count in domain_counts.most_common(25):
        top_domains.append({
            "domain": domain,
            "count": count,
            "providers": domain_providers[domain],
            "blocked_by": domain_blocked.get(domain, set()),
        })

    # Citation rank analysis: blocked vs allowed
    rank_blocked = []
    rank_allowed = []
    for r in citations:
        rank_str = r.get("citation_rank", "").strip()
        blocked_str = r.get("bot_blocked", "").strip()
        if rank_str and blocked_str and rank_str.isdigit():
            rank = int(rank_str)
            if blocked_str == "true":
                rank_blocked.append(rank)
            else:
                rank_allowed.append(rank)

    # Robots.txt fetch status
    status_counts = Counter(
        r.get("robots_status", "unknown") for r in citations
        if r.get("domain", "").strip()
    )

    # Overall blocked count
    all_checked = [r for r in citations if r.get("bot_blocked", "").strip()]
    all_blocked = [r for r in all_checked if r["bot_blocked"] == "true"]

    # Redirect/proxy URL citations (e.g. vertexaisearch.google.com)
    redirect_domains = {"vertexaisearch.cloud.google.com", "vertexaisearch.google.com"}
    redirect_citations = [
        r for r in citations
        if any(rd in r.get("domain", "") for rd in redirect_domains)
    ]
    redirect_providers = Counter(r["provider"] for r in redirect_citations)

    # Empty citation rows (no domain — provider returned no citations)
    all_rows_with_empty = [r for r in rows if not r.get("domain", "").strip()]
    empty_by_provider = Counter(r["provider"] for r in all_rows_with_empty)

    return {
        "total_citations": total_citations,
        "unique_domains": len(unique_domains),
        "unique_prompts": len(unique_prompts),
        "providers": providers_seen,
        "categories": categories_seen,
        "overall_checked": len(all_checked),
        "overall_blocked": len(all_blocked),
        "overall_rate": len(all_blocked) / len(all_checked) * 100 if all_checked else 0,
        "provider_stats": provider_stats,
        "category_stats": category_stats,
        "top_domains": top_domains,
        "rank_blocked": rank_blocked,
        "rank_allowed": rank_allowed,
        "status_counts": status_counts,
        "redirect_citations": len(redirect_citations),
        "redirect_providers": redirect_providers,
        "empty_rows": len(all_rows_with_empty),
        "empty_by_provider": empty_by_provider,
    }


def chart_provider_rates(stats: dict) -> str:
    """Bar chart: violation rate per provider."""
    ps = stats["provider_stats"]
    providers = list(ps.keys())
    rates = [ps[p]["rate"] for p in providers]
    labels = [f"{PROVIDER_DISPLAY_NAMES.get(p, p)}\n({PROVIDER_BOT_MAP.get(p, '?')})" for p in providers]
    colours = [PROVIDER_COLOURS.get(p, GRAY_400) for p in providers]

    fig, ax = plt.subplots(figsize=(7, 4))
    bars = ax.bar(labels, rates, color=colours, width=0.5, edgecolor="white", linewidth=1.5)

    for bar, rate, p in zip(bars, rates, providers):
        count = ps[p]["blocked"]
        total = ps[p]["checked"]
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 0.8,
                f"{rate:.1f}%\n({count}/{total})",
                ha="center", va="bottom", fontsize=10, color=GRAY_700, fontweight="normal")

    ax.set_ylabel("Citations from blocked domains (%)")
    ax.set_title("Violation rate by provider")
    ax.set_ylim(0, max(rates) * 1.35 if rates else 10)
    ax.yaxis.set_major_formatter(mticker.PercentFormatter())
    ax.grid(axis="y", alpha=0.3)

    return fig_to_base64(fig)


def chart_category_rates(stats: dict) -> str:
    """Horizontal bar chart: violation rate by category."""
    cs = stats["category_stats"]
    cats = sorted(cs.keys(), key=lambda c: cs[c]["rate"])
    rates = [cs[c]["rate"] for c in cats]
    labels = [c.replace("_", " ").title() for c in cats]

    fig, ax = plt.subplots(figsize=(7, max(3, len(cats) * 0.45)))
    bars = ax.barh(labels, rates, color=CORAL_500, height=0.6, edgecolor="white", linewidth=1)

    for bar, rate, cat in zip(bars, rates, cats):
        if rate > 0:
            count = cs[cat]["blocked"]
            total = cs[cat]["checked"]
            ax.text(bar.get_width() + 0.3, bar.get_y() + bar.get_height() / 2,
                    f"{rate:.1f}% ({count}/{total})",
                    ha="left", va="center", fontsize=9, color=GRAY_600)

    ax.set_xlabel("Citations from blocked domains (%)")
    ax.set_title("Violation rate by category")
    ax.set_xlim(0, max(rates) * 1.4 if rates and max(rates) > 0 else 10)
    ax.xaxis.set_major_formatter(mticker.PercentFormatter())
    ax.grid(axis="x", alpha=0.3)

    return fig_to_base64(fig)


def chart_top_domains(stats: dict) -> str:
    """Horizontal bar chart: top 20 cited domains, coloured by compliance."""
    domains = stats["top_domains"][:20]
    domains = list(reversed(domains))  # highest at top

    labels = [d["domain"] for d in domains]
    counts = [d["count"] for d in domains]
    colours = [CORAL_600 if d["blocked_by"] else "#22c55e" for d in domains]

    fig, ax = plt.subplots(figsize=(8, max(4, len(domains) * 0.35)))
    bars = ax.barh(labels, counts, color=colours, height=0.6, edgecolor="white", linewidth=1)

    for bar, d in zip(bars, domains):
        suffix = ""
        if d["blocked_by"]:
            suffix = f" [blocks: {', '.join(sorted(d['blocked_by']))}]"
        ax.text(bar.get_width() + 0.3, bar.get_y() + bar.get_height() / 2,
                f"{d['count']}{suffix}",
                ha="left", va="center", fontsize=8, color=GRAY_600)

    ax.set_xlabel("Times cited")
    ax.set_title("Top 20 most-cited domains")
    ax.set_xlim(0, max(counts) * 1.5 if counts else 10)
    ax.grid(axis="x", alpha=0.3)

    # Legend
    from matplotlib.patches import Patch
    legend_elements = [
        Patch(facecolor="#22c55e", label="Allows citing bot"),
        Patch(facecolor=CORAL_600, label="Blocks citing bot"),
    ]
    ax.legend(handles=legend_elements, loc="lower right", fontsize=9, framealpha=0.9)

    return fig_to_base64(fig)


def chart_rank_distribution(stats: dict) -> str:
    """Box/violin comparison of citation ranks: blocked vs allowed."""
    rank_b = stats["rank_blocked"]
    rank_a = stats["rank_allowed"]

    if not rank_b and not rank_a:
        return ""

    fig, ax = plt.subplots(figsize=(6, 4))

    data = []
    labels = []
    colours = []
    if rank_a:
        data.append(rank_a)
        labels.append(f"Allowed\n(n={len(rank_a)})")
        colours.append("#22c55e")
    if rank_b:
        data.append(rank_b)
        labels.append(f"Blocked\n(n={len(rank_b)})")
        colours.append(CORAL_500)

    bp = ax.boxplot(data, tick_labels=labels, patch_artist=True, widths=0.4,
                    medianprops={"color": GRAY_700, "linewidth": 2})
    for patch, color in zip(bp["boxes"], colours):
        patch.set_facecolor(color)
        patch.set_alpha(0.6)

    ax.set_ylabel("Citation rank (1 = top)")
    ax.set_title("Citation rank: blocked vs allowed sources")
    ax.invert_yaxis()
    ax.grid(axis="y", alpha=0.3)

    return fig_to_base64(fig)


def generate_html(stats: dict, input_path: str, generated_at: str) -> str:
    """Build the full self-contained HTML report."""
    setup_plot_style()

    # Generate charts
    img_provider = chart_provider_rates(stats)
    img_category = chart_category_rates(stats)
    img_domains = chart_top_domains(stats)
    img_ranks = chart_rank_distribution(stats)

    # Build category table rows
    cat_rows = ""
    for cat in sorted(stats["category_stats"].keys()):
        cs = stats["category_stats"][cat]
        cat_rows += f"""
            <tr>
                <td>{cat.replace("_", " ").title()}</td>
                <td>{cs["citations"]}</td>
                <td>{cs["domains"]}</td>
                <td>{cs["checked"]}</td>
                <td>{cs["blocked"]}</td>
                <td><strong>{cs["rate"]:.1f}%</strong></td>
            </tr>"""

    # Build top violators table (blocked domains only)
    violator_rows = ""
    violators = [d for d in stats["top_domains"] if d["blocked_by"]]
    for d in violators[:15]:
        blocked_providers = ", ".join(
            f"{PROVIDER_DISPLAY_NAMES.get(p, p)} ({PROVIDER_BOT_MAP.get(p, '?')})" for p in sorted(d["blocked_by"])
        )
        violator_rows += f"""
            <tr>
                <td>{d["domain"]}</td>
                <td>{d["count"]}</td>
                <td>{blocked_providers}</td>
            </tr>"""

    # Provider headline cards
    provider_cards = ""
    for p in sorted(stats["provider_stats"].keys()):
        ps = stats["provider_stats"][p]
        bot = PROVIDER_BOT_MAP.get(p, "?")
        colour = PROVIDER_COLOURS.get(p, GRAY_400)
        provider_cards += f"""
            <div class="metric-card" style="border-left: 4px solid {colour}">
                <div class="metric-label">{PROVIDER_DISPLAY_NAMES.get(p, p)} ({bot})</div>
                <div class="metric-value" style="color: {colour}">{ps["rate"]:.1f}%</div>
                <div class="metric-detail">{ps["blocked"]}/{ps["checked"]} citations from blocked domains</div>
            </div>"""

    # Robots status breakdown
    status_total = sum(stats["status_counts"].values())
    success_pct = stats["status_counts"].get("success", 0) / status_total * 100 if status_total else 0
    fetch_err = stats["status_counts"].get("fetcherror", 0)

    # Precompute nuance section values
    openai_empty = stats["empty_by_provider"].get("openai", 0)
    openai_cited = stats["provider_stats"].get("openai", {}).get("total", 0)
    openai_total_calls = openai_cited + openai_empty

    rank_section = ""
    if img_ranks:
        avg_blocked = sum(stats["rank_blocked"]) / len(stats["rank_blocked"]) if stats["rank_blocked"] else 0
        avg_allowed = sum(stats["rank_allowed"]) / len(stats["rank_allowed"]) if stats["rank_allowed"] else 0
        rank_section = f"""
        <section>
            <h2>Citation Rank: Blocked vs Allowed</h2>
            <p>Do blocked sources appear at higher or lower citation ranks? Lower rank numbers mean the source was cited earlier (more prominently) in the response.</p>
            <img src="data:image/png;base64,{img_ranks}" alt="Citation rank distribution">
            <p class="detail">Mean rank for allowed sources: <strong>{avg_allowed:.1f}</strong>. Mean rank for blocked sources: <strong>{avg_blocked:.1f}</strong>.</p>
        </section>"""

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>LLM Citation Compliance Report - OpenAttribution</title>
    <meta name="description" content="How often do AI search engines cite sources that have blocked their crawler? Data from {stats['total_citations']} citations across {stats['unique_domains']} domains.">
    <link rel="icon" type="image/svg+xml" href="https://openattribution.org/favicon.svg">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600&display=swap" rel="stylesheet">
    <style>
        :root {{
            --coral-50: {CORAL_50};
            --coral-100: {CORAL_100};
            --coral-500: {CORAL_500};
            --coral-600: {CORAL_600};
            --amber-50: {AMBER_50};
            --amber-100: {AMBER_100};
            --amber-500: {AMBER_500};
            --amber-600: {AMBER_600};
            --gray-200: {GRAY_200};
            --gray-400: {GRAY_400};
            --gray-600: {GRAY_600};
            --gray-700: {GRAY_700};
            --cream: {CREAM};
        }}

        * {{ margin: 0; padding: 0; box-sizing: border-box; }}

        body {{
            font-family: 'Inter', system-ui, sans-serif;
            color: var(--gray-700);
            background: var(--cream);
            -webkit-font-smoothing: antialiased;
            line-height: 1.7;
        }}

        /* Navigation */
        nav {{
            position: sticky;
            top: 0;
            background: rgba(255,255,255,0.85);
            backdrop-filter: blur(12px);
            border-bottom: 1px solid var(--coral-100);
            z-index: 50;
            padding: 1rem 1.5rem;
        }}
        nav .inner {{
            max-width: 900px;
            margin: 0 auto;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }}
        nav .logo {{
            font-size: 1.25rem;
            font-weight: 300;
            letter-spacing: -0.02em;
        }}
        nav .logo span {{ color: var(--coral-600); }}
        nav a {{
            color: var(--gray-600);
            text-decoration: none;
            font-weight: 300;
            font-size: 0.875rem;
        }}
        nav a:hover {{ color: var(--coral-600); }}

        /* Hero */
        .hero {{
            background: linear-gradient(135deg, var(--coral-50), var(--cream), var(--amber-50));
            padding: 5rem 1.5rem 3rem;
            text-align: center;
        }}
        .hero h1 {{
            font-size: 2.5rem;
            font-weight: 300;
            line-height: 1.2;
            max-width: 700px;
            margin: 0 auto 1rem;
        }}
        .hero h1 em {{
            color: var(--coral-600);
            font-style: normal;
            font-weight: 400;
        }}
        .hero .subtitle {{
            font-size: 1.125rem;
            font-weight: 300;
            color: var(--gray-600);
            max-width: 600px;
            margin: 0 auto 2rem;
        }}
        .hero .date {{
            font-size: 0.8125rem;
            color: var(--gray-400);
            font-weight: 300;
        }}

        /* Content */
        .content {{
            max-width: 900px;
            margin: 0 auto;
            padding: 0 1.5rem 4rem;
        }}

        section {{
            margin-top: 3rem;
        }}
        section h2 {{
            font-size: 1.75rem;
            font-weight: 400;
            margin-bottom: 1rem;
            color: #1f2937;
        }}
        section p {{
            font-weight: 300;
            margin-bottom: 1rem;
            color: var(--gray-600);
        }}
        section p strong {{
            font-weight: 500;
            color: var(--gray-700);
        }}

        /* Metric cards grid */
        .metrics {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
            gap: 1rem;
            margin: 2rem 0;
        }}
        .metric-card {{
            background: white;
            border-radius: 0.75rem;
            padding: 1.5rem;
            box-shadow: 0 1px 3px rgba(0,0,0,0.06);
        }}
        .metric-label {{
            font-size: 0.8125rem;
            font-weight: 400;
            color: var(--gray-600);
            margin-bottom: 0.25rem;
        }}
        .metric-value {{
            font-size: 2rem;
            font-weight: 400;
            line-height: 1.2;
        }}
        .metric-detail {{
            font-size: 0.75rem;
            font-weight: 300;
            color: var(--gray-400);
            margin-top: 0.25rem;
        }}

        /* Headline stat row */
        .headline-stats {{
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 1rem;
            margin: 2rem 0;
        }}
        .headline-stat {{
            text-align: center;
            padding: 1.25rem;
            background: white;
            border-radius: 0.75rem;
            box-shadow: 0 1px 3px rgba(0,0,0,0.06);
        }}
        .headline-stat .number {{
            font-size: 1.75rem;
            font-weight: 400;
            color: var(--coral-600);
        }}
        .headline-stat .label {{
            font-size: 0.75rem;
            font-weight: 300;
            color: var(--gray-600);
            margin-top: 0.25rem;
        }}

        /* Charts */
        img {{
            width: 100%;
            max-width: 750px;
            display: block;
            margin: 1.5rem auto;
            border-radius: 0.75rem;
            box-shadow: 0 1px 3px rgba(0,0,0,0.06);
        }}

        /* Tables */
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 1.5rem 0;
            font-size: 0.875rem;
        }}
        th {{
            font-weight: 400;
            text-align: left;
            padding: 0.75rem 1rem;
            border-bottom: 2px solid var(--coral-100);
            color: #1f2937;
        }}
        td {{
            padding: 0.75rem 1rem;
            border-bottom: 1px solid #f3f4f6;
            font-weight: 300;
        }}
        tr:hover td {{
            background: var(--coral-50);
        }}
        td strong {{
            font-weight: 500;
        }}

        .detail {{
            font-size: 0.8125rem;
            color: var(--gray-400);
        }}

        /* Callout box */
        .callout {{
            background: linear-gradient(135deg, var(--coral-50), var(--amber-50));
            border-left: 4px solid var(--coral-600);
            border-radius: 0 0.75rem 0.75rem 0;
            padding: 1.5rem 2rem;
            margin: 2rem 0;
        }}
        .callout h3 {{
            font-weight: 400;
            font-size: 1.125rem;
            margin-bottom: 0.5rem;
        }}
        .callout p {{
            margin-bottom: 0.5rem;
        }}
        .callout p:last-child {{
            margin-bottom: 0;
        }}

        /* Footer */
        footer {{
            background: #111827;
            color: var(--gray-400);
            padding: 3rem 1.5rem;
            text-align: center;
            font-size: 0.8125rem;
            font-weight: 300;
        }}
        footer a {{
            color: var(--coral-500);
            text-decoration: none;
        }}
        footer a:hover {{
            text-decoration: underline;
        }}

        /* Accordions (details/summary) */
        .accordion-group {{
            margin: 2rem 0;
        }}
        details {{
            padding: 1.25rem 1.5rem;
            border-radius: 0.75rem;
            cursor: pointer;
            transition: box-shadow 0.2s ease;
            margin-bottom: 0.75rem;
        }}
        details:hover {{
            box-shadow: 0 2px 8px rgba(0,0,0,0.06);
        }}
        details.coral {{
            background: linear-gradient(135deg, var(--coral-50), var(--amber-50));
        }}
        details.amber {{
            background: linear-gradient(135deg, var(--amber-50), var(--coral-50));
        }}
        summary {{
            font-weight: 400;
            font-size: 1.125rem;
            color: #1f2937;
            list-style: none;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }}
        summary::-webkit-details-marker {{
            display: none;
        }}
        summary::after {{
            content: "+";
            font-size: 1.25rem;
            font-weight: 300;
            color: var(--gray-400);
            transition: transform 0.2s ease;
        }}
        details[open] summary::after {{
            content: "\\2212";
        }}
        details .accordion-body {{
            margin-top: 1rem;
            font-weight: 300;
            color: var(--gray-600);
            line-height: 1.7;
        }}
        details .accordion-body p {{
            margin-bottom: 0.75rem;
        }}
        details .accordion-body p:last-child {{
            margin-bottom: 0;
        }}

        /* Responsive */
        @media (max-width: 640px) {{
            .hero h1 {{ font-size: 1.75rem; }}
            .metrics {{ grid-template-columns: 1fr; }}
            .headline-stats {{ grid-template-columns: repeat(2, 1fr); }}
        }}
    </style>
</head>
<body>
    <nav>
        <div class="inner">
            <div class="logo"><span>Open</span>Attribution</div>
            <div>
                <a href="https://openattribution.org">Home</a>
                &nbsp;&nbsp;
                <a href="https://openattribution.org/policycheck/">PolicyCheck</a>
                &nbsp;&nbsp;
                <a href="https://openattribution.org/blog/">Blog</a>
            </div>
        </div>
    </nav>

    <div class="hero">
        <h1>LLM Citation <em>Compliance</em> Report</h1>
        <p class="subtitle">When developers build AI agents with web search, the foundation model chooses what to cite. How often do those citations come from sources that have asked not to be crawled?</p>
        <p class="date">Data collected {generated_at} &middot; {stats["total_citations"]} citations &middot; {stats["unique_domains"]} domains</p>
    </div>

    <div class="content">
        <section>
            <h2>Headlines</h2>

            <div class="headline-stats">
                <div class="headline-stat">
                    <div class="number">{stats["unique_prompts"]}</div>
                    <div class="label">Prompts</div>
                </div>
                <div class="headline-stat">
                    <div class="number">{stats["total_citations"]}</div>
                    <div class="label">Citations</div>
                </div>
                <div class="headline-stat">
                    <div class="number">{stats["unique_domains"]}</div>
                    <div class="label">Unique domains</div>
                </div>
                <div class="headline-stat">
                    <div class="number">{stats["overall_rate"]:.1f}%</div>
                    <div class="label">Overall violation rate</div>
                </div>
                <div class="headline-stat">
                    <div class="number">{stats["overall_blocked"]}</div>
                    <div class="label">Blocked citations</div>
                </div>
                <div class="headline-stat">
                    <div class="number">{len(stats["categories"])}</div>
                    <div class="label">Categories</div>
                </div>
            </div>
        </section>

        <section>
            <h2>Why This Matters</h2>
            <div class="accordion-group">
                <details class="coral">
                    <summary>For agent builders and developers</summary>
                    <div class="accordion-body">
                        <p>These results come from the same APIs that any developer uses to build AI-powered products &mdash; OpenAI's Responses API, Google's Gemini API, Perplexity's Sonar API &mdash; all with web search enabled. When you integrate one of these APIs into an agent, app, or workflow, the foundation model decides which sources to cite. You have no visibility into whether those sources have consented to being crawled by that provider.</p>
                        <p>A publisher sets <code>robots.txt</code> to block GPTBot. You call the OpenAI API with web search. The API returns a citation from that publisher. Your agent surfaces it to the end user. At no point in this chain did anyone check whether the source said yes. You inherit a compliance posture you cannot inspect, from a foundation model you do not control, citing sources whose preferences you have no way to verify.</p>
                    </div>
                </details>
                <details class="amber">
                    <summary>For brands considering AI search visibility</summary>
                    <div class="accordion-body">
                        <p>If you are evaluating publishers as partners, or assessing where your brand appears in AI search results, the compliance status of cited sources matters. A publisher that blocks AI crawlers but still appears in AI-generated answers has an unresolved tension in their content strategy &mdash; and any partnership built on that visibility sits on uncertain ground.</p>
                        <p>Understanding which sources are cited in compliance with their own stated policies, and which are not, helps you make better decisions about where to invest in content partnerships and where AI-driven traffic is sustainable.</p>
                    </div>
                </details>
                <details class="coral">
                    <summary>For publishers</summary>
                    <div class="accordion-body">
                        <p>If you have set <code>robots.txt</code> to block an AI provider's crawler, you have stated your position clearly. This data shows whether that preference is being respected when that provider generates answers with web search. Your content may still be surfaced to users through AI search despite your explicit opt-out.</p>
                        <p>This is not about whether <code>robots.txt</code> is legally enforceable &mdash; it is about whether the signals publishers already use are having their intended effect, and where the gaps are.</p>
                    </div>
                </details>
            </div>
        </section>

        <section>
            <h2>Violation Rate by Provider</h2>
            <p>A "violation" occurs when an AI search engine cites a domain whose <code>robots.txt</code> explicitly blocks that provider's crawler bot. This does not imply illegality &mdash; it means the cited source has expressed a preference not to be crawled by that bot.</p>

            <div class="metrics">
                {provider_cards}
            </div>

            <img src="data:image/png;base64,{img_provider}" alt="Violation rate by provider">
        </section>

        <section>
            <h2>Violation Rate by Category</h2>
            <p>Different content categories show different violation rates. Categories with high-value editorial content (news, health, finance) tend to have higher blocking rates.</p>
            <img src="data:image/png;base64,{img_category}" alt="Violation rate by category">
        </section>

        <section>
            <h2>Category Breakdown</h2>
            <table>
                <thead>
                    <tr>
                        <th>Category</th>
                        <th>Citations</th>
                        <th>Domains</th>
                        <th>Checked</th>
                        <th>Blocked</th>
                        <th>Rate</th>
                    </tr>
                </thead>
                <tbody>
                    {cat_rows}
                </tbody>
            </table>
        </section>

        <section>
            <h2>Top Cited Domains</h2>
            <p>The 20 most frequently cited domains across all providers and prompts, coloured by whether the domain blocks the citing provider's bot.</p>
            <img src="data:image/png;base64,{img_domains}" alt="Top 20 most-cited domains">
        </section>

        {rank_section}

        <section>
            <h2>Top Violators</h2>
            <p>Domains that were both frequently cited <em>and</em> block the citing provider's crawler.</p>
            <table>
                <thead>
                    <tr>
                        <th>Domain</th>
                        <th>Times cited</th>
                        <th>Blocks</th>
                    </tr>
                </thead>
                <tbody>
                    {violator_rows if violator_rows else "<tr><td colspan='3'>No violations detected in this dataset.</td></tr>"}
                </tbody>
            </table>
        </section>

        <div class="callout">
            <h3>Methodology</h3>
            <p>Every citation in this report comes directly from the providers' own APIs with their built-in web search tools enabled &mdash; not from the consumer web portals (chatgpt.com, gemini.google.com, perplexity.ai). Specifically: OpenAI's Responses API with <code>web_search_preview</code>, Google's Gemini API with <code>google_search</code> grounding, and Perplexity's Sonar API (which includes web search by default). Each prompt was submitted through these official APIs and the cited URLs were extracted programmatically exactly as returned &mdash; no scraping, no browser automation, no modification of results.</p>
            <p>Each unique cited domain was then checked against the <a href="https://github.com/openattribution-org/policycheck">PolicyCheck</a> server, which fetches and parses the domain's <code>robots.txt</code> to determine per-bot access status for 26 known AI crawlers.</p>
            <p>A citation is flagged as "blocked" when the cited domain's <code>robots.txt</code> disallows the citing provider's primary crawler: GPTBot for OpenAI, Google-Extended for Gemini, PerplexityBot for Perplexity.</p>
            <p>Robots.txt was successfully fetched for <strong>{success_pct:.0f}%</strong> of citation lookups ({fetch_err} fetch errors). Domains that returned fetch errors are excluded from violation calculations.</p>
            <p class="detail">This is a point-in-time snapshot. Robots.txt policies and LLM citation behaviour change over time. This report should not be interpreted as evidence of illegality &mdash; robots.txt is advisory, not legally binding in all jurisdictions.</p>
        </div>

        <section>
            <h2>Limitations and Nuance</h2>
            <p>Robots.txt is a useful public signal, but it does not tell the whole story. Several patterns in this data illustrate why.</p>

            <div class="accordion-group">
                <details class="coral">
                    <summary>Data licensing deals bypass robots.txt</summary>
                    <div class="accordion-body">
                        <p>Reddit blocks every crawler with a blanket <code>User-agent: * / Disallow: /</code>. Yet Gemini cited reddit.com 14 times in this dataset &mdash; and it was the <em>only</em> provider to do so. OpenAI and Perplexity, who do not have a data deal with Reddit, never cited it.</p>
                        <p>Google has a reported data licensing agreement with Reddit. The content reaches Gemini through a direct feed, not through crawling. The <code>robots.txt</code> is technically blocking Google-Extended, but that is irrelevant when the data arrives via a commercial API.</p>
                        <p>This means a "blocked" status in <code>robots.txt</code> does not necessarily mean the provider lacks access. Private data deals create a layer of access that is invisible to any public compliance check. For agent builders and publishers alike, the observable signal (robots.txt) and the actual access (licensed feed) can diverge completely.</p>
                    </div>
                </details>
                <details class="amber">
                    <summary>Proxy and redirect URLs obscure real sources</summary>
                    <div class="accordion-body">
                        <p>Gemini's API sometimes returns citations pointing to <code>vertexaisearch.cloud.google.com/grounding-api-redirect/...</code> instead of the actual source URL. {stats["redirect_citations"]} citations in this dataset were redirect URLs rather than real domains. These cannot be meaningfully checked for compliance because they are opaque intermediaries &mdash; the real source domain is hidden behind Google's redirect layer.</p>
                        <p>This is a transparency gap: the agent builder receives a citation but cannot determine the true source without following the redirect. It makes independent compliance verification harder.</p>
                    </div>
                </details>
                <details class="coral">
                    <summary>Missing citations: not all API calls produce sources</summary>
                    <div class="accordion-body">
                        <p>{stats["empty_rows"]} API responses in this dataset returned no citations at all despite web search being enabled. {openai_empty} of these were from OpenAI, which produced citations for only {openai_cited} of its {openai_total_calls} responses. By contrast, Gemini and Perplexity almost always returned cited sources.</p>
                        <p>This does not mean OpenAI's answers were unsourced &mdash; the model may have used web search internally but not surfaced the citations in the API response. However, it means the compliance picture for OpenAI is based on a smaller sample of citations relative to the number of queries.</p>
                    </div>
                </details>
            </div>
        </section>

        <section>
            <h2>About</h2>
            <p>This report was produced by <a href="https://openattribution.org">OpenAttribution</a> using <a href="https://github.com/openattribution-org/policycheck">PolicyCheck</a> for compliance checking. All data, prompts, and code are published in the <a href="https://github.com/openattribution-org/policycheck/tree/main/research">research directory</a> for reproducibility.</p>
            <p class="detail">Source data: <code>{input_path}</code></p>
        </section>
    </div>

    <footer>
        <p>&copy; 2026 <a href="https://openattribution.org">OpenAttribution</a>. All rights reserved.</p>
        <p style="margin-top: 0.5rem;">Built with <a href="https://github.com/openattribution-org/policycheck">PolicyCheck</a></p>
    </footer>
</body>
</html>"""

    return html


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate HTML compliance report from enriched citations CSV"
    )
    parser.add_argument(
        "input_csv",
        type=Path,
        help="Path to enriched_citations CSV",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Output HTML path (default: output/report_{timestamp}.html)",
    )
    args = parser.parse_args()

    if not args.input_csv.exists():
        print(f"error: {args.input_csv} not found")
        sys.exit(1)

    output_dir = Path(__file__).parent / "output"
    output_dir.mkdir(exist_ok=True)

    if args.output:
        output_path = args.output
    else:
        timestamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
        output_path = output_dir / f"report_{timestamp}.html"

    print(f"Reading {args.input_csv}...")
    rows = load_csv(args.input_csv)
    print(f"  {len(rows)} rows loaded")

    print("Computing statistics...")
    stats = compute_stats(rows)

    generated_at = datetime.now(timezone.utc).strftime("%d %B %Y")
    print("Generating report...")
    html = generate_html(stats, str(args.input_csv), generated_at)

    with open(output_path, "w", encoding="utf-8") as f:
        f.write(html)

    print(f"\nReport written to {output_path}")
    print(f"  {stats['total_citations']} citations, {stats['unique_domains']} domains")
    print(f"  Overall violation rate: {stats['overall_rate']:.1f}%")


if __name__ == "__main__":
    main()
