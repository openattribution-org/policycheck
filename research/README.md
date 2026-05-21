# Citation Compliance Research

Pipeline that runs prompts through AI search providers, extracts cited URLs, and checks each cited domain's robots.txt to flag citations from sources that block the citing provider's crawler.

See `../REPORT_BRIEF.md` for the full brief and `output/report_*.html` for prior runs.

## Providers

| Provider | Model | Bot checked |
|----------|-------|-------------|
| OpenAI | gpt-5-mini | GPTBot |
| Anthropic | claude-sonnet-4-6 | ClaudeBot |
| Google | gemini-3-flash-preview | Google-Extended |
| Perplexity | sonar | PerplexityBot |

## Setup

```bash
# Install deps (uv handles the venv)
uv sync

# API keys in .env (see .env.example)
cp .env.example .env
# fill in OPENAI_API_KEY, ANTHROPIC_API_KEY, GEMINI_API_KEY, PERPLEXITY_API_KEY

# Build PolicyCheck binary (one-off, ~3 min)
cargo build --release -p policycheck --manifest-path ../Cargo.toml
```

## Run

Three steps, in order. Each writes to `output/` with a UTC timestamp.

### 1. Start a local PolicyCheck server

Run this in a separate terminal and leave it up for the duration of the run.

```bash
../target/release/policycheck serve --port 3001
```

The local server avoids datacenter IP blocks that affect the public Fly.io deployment, which otherwise loses a share of citations to fetch errors when publishers block Fly.io egress.

### 2. Collect citations

```bash
# Dry run first — prints the plan without spending
uv run python run_citations.py --dry-run

# Full run — 330 prompts × 4 providers × 3 runs = 3,960 API calls
uv run python run_citations.py --runs 3

# Or scope smaller first
uv run python run_citations.py --limit 10 --runs 1
```

Flags: `--providers openai,anthropic` to subset. `--delay` controls inter-prompt pause (default 1s).

### 3. Enrich and report

```bash
# Pick the latest raw_citations file
RAW=$(ls -t output/raw_citations_*.csv | head -1)

# Enrich against the local server
uv run python enrich_citations.py "$RAW" --server http://localhost:3001

# Generate the HTML report
ENRICHED=$(ls -t output/enriched_citations_*.csv | head -1)
uv run python generate_report.py "$ENRICHED"
```

## Cost estimate

Full run (330 prompts × 4 providers × 3 runs):

| Provider | Approx |
|----------|--------|
| OpenAI (gpt-5-mini + web_search) | ~$60 |
| Anthropic (sonnet 4.6 + web_search) | ~$90 |
| Gemini (grounded search) | ~$35 |
| Perplexity Sonar | ~$8 |
| **Total** | **~$190** |

PolicyCheck calls are free (local).
