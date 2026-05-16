# Published audit datasets

Tracked snapshots of completed audit runs. Working papers and reports reference these files directly — they don't move once published.

Ad-hoc runs write to `../output/` (gitignored) and only graduate into this directory when they back a published artefact.

## May 2026 run

Used by [Measuring content influence in AI assistants](https://openattribution.org/research/measuring-content-influence-in-ai-assistants) and the [LLM citation compliance report - May 2026](https://openattribution.org/research/citation-compliance-may-2026/).

- `raw_citations_20260514_193439.csv` - 24,127 citations from 330 prompts × 4 providers × 3 runs. Pre-enrichment.
- `enriched_citations_20260515_115259.csv` - same citations with per-bot robots.txt match results from PolicyCheck (training crawler and live-search bot for each provider).

Columns in the enriched CSV: `category, prompt, provider, model, run_number, citation_rank, citation_url, domain, citation_title, timestamp, training_bot, training_bot_blocked, live_bot, live_bot_blocked, bot_blocked, robots_status`.
