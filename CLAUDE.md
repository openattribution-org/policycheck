# PolicyCheck

Web attribution and compliance scanner. Checks robots.txt, RSL licenses, and TDM policies for URLs. Part of [OpenAttribution](https://openattribution.org).

## Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 2021 edition (1.75+) |
| HTTP client | reqwest (async, rustls-tls) |
| CLI | clap (derive) |
| HTTP server | axum + tower |
| robots.txt parser | texting_robots |
| Serialisation | serde + serde_json |
| Tables | comfy-table |
| CSV | csv crate |
| Error handling | anyhow |

## Architecture

Single binary, seven modules:

```
src/
  main.rs        — CLI entry point (clap), dispatches to analyze or serve
  ai_crawlers.rs — Canonical list of 26 AI crawlers (GPTBot, ClaudeBot, etc.)
  analyzer.rs    — Core logic: parse robots.txt, extract licenses, evaluate TDM, analyze AI bots
  fetcher.rs     — HTTP fetching for robots.txt and /.well-known/tdmrep.json
  models.rs      — Data types: AnalysisResult, TdmPolicy, BotAnalysisResult, request/response shapes
  output.rs      — Formatters: table, JSON, CSV (with AI bot columns), compact text
  server.rs      — Axum HTTP API (GET /health, POST /analyze)
```

No workspaces, no proc macros, no feature flags. Keep it simple.

## Commands

```bash
# Build
cargo build                    # Debug
cargo build --release          # Optimised (LTO, strip)

# Test
cargo test                     # All tests
cargo test -- --nocapture      # With stdout

# Run - Single URL
cargo run -- analyze --url https://www.nytimes.com
cargo run -- analyze --url https://github.com --format json

# Run - Bulk analysis with CSV export (advertiser use case)
cargo run -- analyze --csv publishers.csv --format csv --output results.csv

# Run - HTTP server
cargo run -- serve --port 3000

# Lint
cargo clippy
cargo fmt --check
```

## Development Rules

- **Test-driven** — write the test first, then implement
- **Keep it flat** — no nested modules, no abstractions for one use
- **anyhow for errors** — `Result<T>` everywhere, `context()` for wrapping
- **No unsafe** — there's no reason for it here
- **No unnecessary dependencies** — think hard before adding a crate

## Testing

Tests live alongside code in `#[cfg(test)] mod tests` blocks. Currently in `analyzer.rs`:

- Unit tests for robots.txt parsing (user agents, paths, licenses)
- RSL licence extraction (global, group-scoped, precedence, absolute URI validation)
- TDM pattern matching (wildcards, end markers, complex patterns)
- TDM rule evaluation (async tests with `#[tokio::test]`)

When adding features:
1. Write `#[test]` or `#[tokio::test]` in the relevant module
2. Make it fail
3. Implement until green
4. `cargo clippy` + `cargo fmt`

## Standards Implemented

| Standard | Status | Where |
|----------|--------|-------|
| RFC 9309 (Robots Exclusion Protocol) | Done | `analyzer.rs` via `texting_robots` |
| RSL (Responsible Sourcing License) | Done | `analyzer.rs::extract_licenses()` |
| W3C TDMRep | Done | `fetcher.rs::fetch_tdm_policy()`, `analyzer.rs::evaluate_tdm_policy()` |
| AI Crawler Analysis | Done | `ai_crawlers.rs`, `analyzer.rs::analyze_ai_bots()` |
| RFC 9116 (security.txt) | Planned | — |
| RFC 8615 (Well-Known URIs) | Planned | — |

## Key Design Decisions

- **texting_robots for parsing** — battle-tested against 34M+ real robots.txt files, handles edge cases
- **AI bot analysis** — canonical list of 26 known AI crawlers, per-bot status (Blocked/Allowed)
- **CSV format with bot columns** — major AI bots as columns for advertiser analysis (Excel/Sheets ready)
- **Simplified bot status** — only Blocked or Allowed (not "Not Mentioned" which confuses users)
- **RSL licence precedence** — group-scoped overrides global (matches RSL spec)
- **TDM pattern matching** — supports `*` wildcards and `$` end markers per W3C TDMRep
- **500KB robots.txt limit** — follows Google's recommendation
- **10s HTTP timeout** — fail fast, don't hang on unresponsive hosts
- **Concurrent analysis** — `tokio::spawn` per URL for parallel fetching
- **rustls-tls** — reqwest uses `rustls-tls` (not `native-tls`) to avoid OpenSSL C dependency; required for cross-compiling aarch64 Linux in CI
- **Server limits** — 1MB body limit, max 100 URLs per request

## CI/CD

GitHub Actions workflows in `.github/workflows/`:
- `ci.yml` — build + test on push/PR
- `release.yml` — binary releases via `softprops/action-gh-release@v2`
- `docs.yml` — documentation

### Cross-compilation

The aarch64 Linux build cross-compiles on x86_64 runners. This requires:
- `gcc-aarch64-linux-gnu` installed on the runner
- `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc` set during build
- `aarch64-linux-gnu-strip` used instead of `strip` for the binary
- `rustls-tls` instead of `native-tls` (OpenSSL can't cross-compile without a sysroot)

## Examples and Testing

**IMPORTANT: Use real-world examples, not example.com**

Good examples:
- ✅ **https://www.nytimes.com** — comprehensive AI bot blocking, thorough robots.txt
- ✅ **https://github.com** — permissive, allows all AI bots
- ✅ **https://techcrunch.com** — selective blocking

Bad examples:
- ❌ **example.com** — generic, boring, not representative of real-world usage

When writing docs or tests, use NYTimes as the primary example (shows comprehensive blocking) and GitHub as the contrast (shows permissive approach).

## AI Bot Analysis

Major feature for two use cases:

1. **Publishers** — Check which AI training bots can access content
2. **Advertisers** — Evaluate publisher partnerships based on AI search visibility

The tool analyzes 26 known AI crawlers categorized as:
- Training bots (GPTBot, ClaudeBot, CCBot, anthropic-ai, etc.)
- Search bots (OAI-SearchBot, PerplexityBot, etc.)
- User-triggered bots (ChatGPT-User, etc.)

CSV output includes major AI bots as columns for easy Excel/Sheets analysis.

## Tone

- British English in docs and comments
- Keep README user-facing and practical
- No marketing fluff in code comments
