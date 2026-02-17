# OpenCite + PolicyCheck Integration Plan

## Vision

Combine OpenCite's LLM citation tracking with PolicyCheck's compliance checking to build a public dashboard that quantifies how often AI search engines violate publisher policies and cite low-quality sources.

**Three audiences:**

1. **Publishers** — "Your robots.txt says no, but ChatGPT cited you 47 times this week"
2. **Brands** — "When people ask about your product, 30% of AI-cited sources are spam"
3. **Industry** — Public compliance scorecard across ChatGPT, Gemini, Perplexity

## Architecture

```
┌─────────────────────────────────────────────────────┐
│              Scheduled Prompt Runs                   │
│  Curated prompt sets by category                    │
│  Multiple LLMs (ChatGPT, Gemini, Perplexity)       │
│  Rotating personas and locations                    │
└──────────────────────┬──────────────────────────────┘
                       │
          ┌────────────▼────────────┐
          │   OpenCite Bulk Runner  │
          │   Citations + ranks     │
          └────────────┬────────────┘
                       │
          ┌────────────▼────────────┐
          │   PolicyCheck /analyze  │
          │   robots.txt + signals  │
          └────────────┬────────────┘
                       │
          ┌────────────▼────────────┐
          │   Combined Dataset      │
          │   citation + compliance │
          └────────────┬────────────┘
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
  Publisher View   Brand View   Industry Scorecard
```

## Phase 1: PR to OpenCite (PolicyCheck enrichment module)

**Goal:** Add a post-processing step to OpenCite that enriches citation data with PolicyCheck compliance info.

**Deliverables:**

### 1a. Python module: `compliance.py`

New file in `modular_bulk_prompt_runner/github_scripts/reports/compliance.py`

Responsibilities:
- Take deduplicated domain list from OpenCite's enriched DataFrame
- Batch POST to PolicyCheck API (`https://policycheck.openattribution.org/analyze`)
- Cache results per domain (robots.txt doesn't change per-minute)
- Return compliance data merged back into the DataFrame

New columns added to the enriched DataFrame:
- `pc_robots_found` — bool, whether robots.txt exists
- `pc_gptbot_status` — "Blocked" | "Allowed"
- `pc_claudebot_status` — "Blocked" | "Allowed"
- `pc_google_extended_status` — "Blocked" | "Allowed"
- `pc_perplexitybot_status` — "Blocked" | "Allowed"
- `pc_content_signal_search` — "yes" | "no" | null
- `pc_content_signal_ai_input` — "yes" | "no" | null
- `pc_content_signal_ai_train` — "yes" | "no" | null
- `pc_rsl_license_count` — int
- `pc_crawl_delay` — int | null
- `pc_violation` — bool (was this domain cited by an LLM whose crawler it blocks?)

The `pc_violation` column is the money metric. It cross-references:
- Which LLM generated the citation (from OpenCite's `provider` column)
- Which bot that LLM uses (ChatGPT → GPTBot, Gemini → Google-Extended, etc.)
- Whether that bot is blocked (from PolicyCheck)

### 1b. Notebook cell: `cell_11e_compliance_report.py`

New cell in the OpenCite notebook that:
- Runs compliance enrichment on the master dataset
- Displays an interactive compliance report with:
  - Violation rate per LLM
  - Top cited domains that block the citing LLM
  - Content Signals adoption rate
  - Side-by-side: "cited rank vs. compliance status"

### 1c. PolicyCheck API batch endpoint (optional optimisation)

If needed, add a lightweight endpoint to PolicyCheck:
```
POST /analyze/batch-domains
{
  "domains": ["nytimes.com", "github.com", ...],
  "bots": ["GPTBot", "ClaudeBot", "Google-Extended", "PerplexityBot"]
}
```

Returns only the bot statuses needed, skipping sitemaps/paths/etc. for faster responses.
The existing `/analyze` endpoint already works fine for this, but a slimmer response
would reduce bandwidth for large batches.

## Phase 2: Proof-of-Concept Report

**Goal:** Publish a single compelling report that demonstrates the concept.

### Methodology
1. Curate 50 prompts across 5 categories:
   - Consumer electronics reviews (the Sony WH-1000XM5 use case)
   - News queries ("latest on X")
   - Health information
   - Financial advice
   - Travel recommendations
2. Run each prompt through ChatGPT, Gemini, Perplexity via OpenCite
3. Enrich all cited domains with PolicyCheck compliance data
4. Calculate violation rates and source quality metrics

### Key metrics for the report
- **Violation rate per LLM**: % of citations from domains that block that LLM's crawler
- **Source quality distribution**: domain age, editorial presence indicators
- **Category variation**: which topics have worse compliance?
- **Cross-LLM comparison**: same query, different source quality

### Output
- Blog post with key findings (openattribution.org)
- Full dataset published as CSV on GitHub
- Methodology documented for reproducibility

## Phase 3: Public Dashboard

**Goal:** Always-on dashboard at openattribution.org showing compliance metrics.

### Hosting (free tier strategy)
- **PolicyCheck API**: Already on Fly.io, scales to zero, essentially free
- **OpenCite runs**: Scheduled via GitHub Actions or Colab (API costs are the real expense)
- **Dashboard frontend**: Static site on GitHub Pages or Cloudflare Pages (free)
- **Data storage**: SQLite file in a GitHub repo, or Cloudflare D1 (free tier)
- **Scheduling**: GitHub Actions cron for weekly runs (free for public repos)

### Dashboard views
1. **Industry Scorecard** (public, no login)
   - Compliance rate per LLM, updated weekly
   - Trend lines over time
   - Category breakdowns
2. **Domain Lookup** (public, no login)
   - Enter a domain, see: how often cited, by which LLMs, compliance status
   - "Rolling Stone: cited 83 times by ChatGPT, blocks GPTBot"
3. **Brand Query** (public, rate-limited)
   - Enter a product/brand query
   - See cached results from recent runs, or queue a new run

### Cost estimates
- PolicyCheck API: ~$0 (Fly.io free tier, scales to zero)
- OpenCite LLM calls: ~$5-20/week for 500 prompts across 3 LLMs
- Dashboard hosting: $0 (static site)
- Domain: Already have openattribution.org
- **Total: $20-80/month**, mostly LLM API costs

## Phase 4: Ecosystem

- GitHub Action: "Check your domain's AI citation compliance"
- Publisher badge: "AI Policy Verified by OpenAttribution"
- Weekly email digest: top compliance violations
- API for third-party tools to query compliance data

## LLM-to-Bot Mapping

Critical for computing the `pc_violation` flag:

| LLM Provider | Citation Source (OpenCite) | Crawler Bot (PolicyCheck) |
|-------------|--------------------------|--------------------------|
| ChatGPT (OpenAI) | `openai` provider | GPTBot, OAI-SearchBot |
| Gemini (Google) | `google` provider | Google-Extended, Googlebot |
| Perplexity | `perplexity` provider | PerplexityBot |
| Claude (Anthropic) | future | ClaudeBot, anthropic-ai |
| Copilot (Microsoft) | future | Bingbot |

## Open Questions

1. **Perplexity support in OpenCite?** Currently supports OpenAI + Google. Would need a Perplexity cartridge.
2. **Domain age / quality signals**: PolicyCheck doesn't currently assess source quality (only policy compliance). Should it? Or is that a separate service?
3. **Caching strategy**: How long to cache PolicyCheck results? robots.txt can change, but not minute-to-minute. 24h cache seems reasonable.
4. **Rate limiting the public dashboard**: How to prevent abuse of the domain lookup / brand query features?
5. **Legal considerations**: Is publishing "ChatGPT violated robots.txt X times" legally sensitive? Probably fine as factual reporting, but worth considering.

## Collaboration Model

- **OpenAttribution (PolicyCheck)**: Compliance checking API, publisher/brand framing, dashboard hosting
- **OpenCite (smartaces)**: Citation extraction engine, LLM integration, prompt methodology
- **Joint**: Combined dataset, public report, dashboard

## Next Steps

1. [ ] Reach out to OpenCite maintainer with this plan
2. [ ] Build `compliance.py` module as a PR to OpenCite
3. [ ] Run proof-of-concept with 50 prompts (consumer electronics)
4. [ ] Publish findings as blog post
5. [ ] Build static dashboard MVP
