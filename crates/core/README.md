# policycheck-core

Pure parsing and analysis library for web publisher compliance policies. No network I/O — callers provide raw content, this library parses it.

Part of [PolicyCheck](https://github.com/openattribution-org/policycheck) by [OpenAttribution](https://openattribution.org).

## Standards

| Standard | Module |
|----------|--------|
| RFC 9309 (Robots Exclusion Protocol) | `checks::robots` |
| RSL (Responsible Sourcing License) | `checks::rsl` |
| W3C TDMRep (Text & Data Mining) | `checks::tdm` |
| Cloudflare Content Signals | `checks::content_signals` |
| AI Crawler Analysis (26 bots) | `checks::ai_bots` |

## Usage

```rust
use policycheck_core::PolicyAnalyzer;

let analyzer = PolicyAnalyzer::new("GPTBot".to_string());
let result = analyzer.analyze(
    "https://www.nytimes.com",
    "User-agent: GPTBot\nDisallow: /\n",
    None,
);

assert!(!result.is_path_allowed);
```

Individual check modules are also available directly:

```rust
use policycheck_core::checks;

let rsl = checks::rsl::extract(robots_txt_content, "GPTBot");
let signals = checks::content_signals::extract(robots_txt_content, "*");
```

## WASM

This crate is designed to be WASM-compatible (no filesystem or network dependencies). See `policycheck-wasm` for the browser wrapper.

## Licence

MIT
