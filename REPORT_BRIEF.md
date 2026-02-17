# Brief: LLM Citation Compliance Report

## Goal

Produce a publishable report answering: **"How often do AI search engines cite sources that have explicitly blocked their crawler?"**

The report should be credible enough for tech press and useful enough for publishers and advertisers. It needs real data at scale, not a handful of examples.

## Background

### The problem

When ChatGPT answers a question with web search, it cites sources. Some of those sources have explicitly told OpenAI's crawler (GPTBot) not to access their content via robots.txt. The same applies to Google's Gemini (Google-Extended) and Perplexity (PerplexityBot).

There is no public data on how often this happens.

### The tools

- **PolicyCheck** (https://github.com/openattribution-org/policycheck) — Rust CLI and API that checks robots.txt policies for any URL. Deployed at `https://policycheck.openattribution.org`. Analyses 26 AI crawlers, Content Signals, RSL licences.

- **OpenCite** (https://github.com/smartaces/opencite) — Python tool that runs prompts through LLMs with web search and extracts every cited URL with rank and context.

- **compliance.py** — A module we built that joins the two: takes citation data, checks each cited domain against PolicyCheck, flags violations where the citing LLM's crawler is blocked by the cited domain's robots.txt.

### What exists so far

A working proof-of-concept at `/Users/alexs/Work/opencite/` that:
1. Sends prompts to OpenAI's Responses API with `web_search_preview` tool
2. Extracts citations (URL, title, domain, rank) from responses
3. Batch-checks all cited domains against PolicyCheck API
4. Flags violations and produces a compliance report

The PoC ran 3 prompts and got 18 citations across 7 domains. That's too small to be meaningful. We need 50-100x that.

Key files:
- `/Users/alexs/Work/opencite/test_compliance_real.py` — The working end-to-end script
- `/Users/alexs/Work/opencite/modular_bulk_prompt_runner/github_scripts/reports/compliance.py` — The compliance module
- `/Users/alexs/Work/openattribution-org/policycheck/INTEGRATION_PLAN.md` — Broader integration plan

### LLM-to-crawler mapping

This is critical for violation detection:

| LLM Provider | API provider field | Primary crawler bot |
|-------------|-------------------|-------------------|
| ChatGPT | openai | GPTBot |
| Gemini | google | Google-Extended |
| Perplexity | perplexity | PerplexityBot |

A "violation" = an LLM cited a domain whose robots.txt blocks that LLM's crawler.

## What needs to be built

### 1. Prompt dataset

Create a CSV of 100 prompts across 10 categories:

| Category | Example prompts | Why it matters |
|----------|----------------|----------------|
| Consumer electronics | "sony wh-1000xm5 vs bose qc ultra", "best 4k tv 2025" | Your LinkedIn post started here |
| News/current events | "latest ukraine conflict developments", "US election polls" | News publishers are the most aggressive blockers |
| Health | "ozempic side effects", "best treatment for lower back pain" | High-stakes misinformation risk |
| Finance | "best savings accounts 2025", "should I invest in bitcoin" | Regulated content, trust matters |
| Travel | "best hotels in rome", "cheap flights to tokyo" | Mix of editorial and affiliate sites |
| Food/recipes | "best sourdough bread recipe", "meal prep ideas" | Recipe sites have strong robots.txt |
| Technology | "how to set up a VPN", "best password manager" | Mix of editorial and commercial |
| Sports | "premier league transfer rumours", "nba playoff predictions" | News-adjacent, paywalled content |
| Education | "best online courses for python", "how to learn machine learning" | Mix of free and paywalled |
| Shopping | "best running shoes for flat feet", "affordable standing desk" | Affiliate/review site ecosystem |

10 prompts per category. Prompts should be the kind of thing a normal person would type into ChatGPT — natural language, not keyword queries.

### 2. Runner script

A Python script that:
- Reads the prompt CSV
- Runs each prompt through OpenAI (gpt-4.1 with web_search_preview)
- Optionally runs through Google Gemini too (stretch goal — needs Vertex AI setup)
- Extracts all citations with URL, domain, title, rank, prompt, provider
- Saves raw results as CSV (one row per citation)
- Runs all cited domains through PolicyCheck API (`https://policycheck.openattribution.org/analyze`)
- Joins compliance data to citation data
- Saves enriched CSV and compliance report CSV

The script at `/Users/alexs/Work/opencite/test_compliance_real.py` does most of this already. It needs:
- CSV input instead of hardcoded prompts
- Multiple runs per prompt (run each prompt 3 times to measure citation consistency)
- Rate limiting and error handling for the OpenAI API
- Progress reporting
- Proper output file naming with timestamps

### 3. Report generator

Takes the enriched CSV and produces a static HTML report with:

**Headline metrics:**
- Total prompts run, total citations extracted, unique domains cited
- Overall violation rate (% of citations from domains blocking the LLM's crawler)
- Number of domains blocking at least one AI bot

**Charts/visualisations (use matplotlib or plotly, render to static images or inline HTML):**
- Violation rate by category (bar chart)
- Top 20 most-cited domains with blocked/allowed status (horizontal bar)
- Distribution of citation ranks for violating vs non-violating sources (do blocked sites appear at higher ranks?)
- Bot blocking heatmap: domains × bots matrix showing blocked/allowed
- Content Signals adoption rate across cited domains

**Tables:**
- Top violators: most-cited domains that block GPTBot
- Category breakdown: prompts, citations, unique domains, violation rate per category
- Full domain compliance table (the CSV we already produce, but formatted nicely)

**Narrative sections (can be written manually after data is generated):**
- Methodology: how we ran the prompts, what we checked, what "violation" means
- Key findings: 3-5 bullet points with the most striking results
- Implications: what this means for publishers, advertisers, and LLM companies
- Credits: OpenCite for citation extraction, PolicyCheck for compliance checking

The report should be a single self-contained HTML file that can be hosted on openattribution.org.

### 4. Credibility requirements

This report will be used for PR. It needs to be:

- **Reproducible**: All prompts, raw data, and code published on GitHub
- **Transparent**: Methodology section explaining every step
- **Conservative**: Don't overstate. "X% of citations came from domains that block GPTBot" is a fact. "ChatGPT violates copyright X% of the time" is an interpretation we should avoid.
- **Dated**: This is a snapshot. robots.txt changes. Citation patterns change. Date everything.
- **Properly attributed**: Credit OpenCite, credit PolicyCheck, link to both repos

### 5. Cost estimate

- 100 prompts × 3 runs × ~$0.05 per web search call = ~$15
- PolicyCheck API calls: free (our own server)
- Total: ~$15 in OpenAI API costs

## Technical details

### OpenAI Responses API usage

```python
from openai import OpenAI
client = OpenAI()

response = client.responses.create(
    model="gpt-4.1",
    tools=[{"type": "web_search_preview"}],
    input="your prompt here",
)

# Citations are in response.output[].content[].annotations[]
# Filter for ann.type == "url_citation"
# Each has: ann.url, ann.title
```

### PolicyCheck API usage

```
POST https://policycheck.openattribution.org/analyze
Content-Type: application/json

{
  "urls": ["https://nytimes.com", "https://github.com"],
  "user_agent": "*"
}
```

Returns per-URL: status, ai_bot_analysis (26 bots with blocked/allowed), content signals, RSL licences, crawl delay.

Max 100 URLs per request. Batch domains, not individual citation URLs.

### Violation detection logic

```python
PROVIDER_BOT_MAP = {
    "openai": "GPTBot",
    "google": "Google-Extended",
    "perplexity": "PerplexityBot",
}

# For each citation:
# 1. Get the provider (e.g. "openai")
# 2. Look up the primary bot (e.g. "GPTBot")
# 3. Check if that bot is "blocked" for the cited domain
# 4. If blocked → violation
```

### Datacenter IP blocking caveat

PolicyCheck runs on Fly.io (cloud infrastructure). Some sites block datacenter IPs, which means PolicyCheck cannot fetch their robots.txt. These sites will show as "fetch_error" rather than "blocked" or "allowed". The report should note this limitation and report the number of domains that couldn't be checked.

## Environment

- Python venv at `/Users/alexs/Work/opencite/.venv/` with pandas, requests, openai installed
- OpenAI API key in `~/.zshrc` as `OPENAI_API_KEY` (NOTE: this key has been exposed and needs rotating. Check it's been regenerated before running.)
- PolicyCheck API live at `https://policycheck.openattribution.org`
- compliance.py module at `/Users/alexs/Work/opencite/modular_bulk_prompt_runner/github_scripts/reports/compliance.py`

## Deliverables

1. `prompts.csv` — 100 prompts across 10 categories
2. `run_report.py` — Script that runs prompts and produces raw citation data
3. `generate_report.py` — Script that takes raw data and produces HTML report
4. `raw_citations.csv` — One row per citation with all metadata
5. `compliance_report.csv` — One row per domain with compliance data
6. `report.html` — Self-contained publishable HTML report
7. All hosted in the policycheck repo under a `/report` or `/research` directory

## Credits

- **PolicyCheck** by OpenAttribution (https://openattribution.org) — robots.txt compliance checking
- **OpenCite** by @smartaces (https://opencite.ai) — LLM citation methodology and inspiration
- The compliance module (`compliance.py`) is a contribution to the OpenCite project
