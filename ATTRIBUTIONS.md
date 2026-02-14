# Attributions

PolicyCheck is built on the shoulders of giants. We gratefully acknowledge the following open-source projects:

## Core Dependencies

### texting_robots
- **Purpose**: Robust robots.txt parsing, battle-tested against 34M+ real-world files
- **Author**: Stephen Merity ([@Smerity](https://github.com/Smerity))
- **License**: MIT OR Apache-2.0
- **Repository**: https://github.com/Smerity/texting_robots
- **Why we use it**: Industry-standard parser with comprehensive edge case handling

### reqwest
- **Purpose**: HTTP client for fetching robots.txt and .well-known resources
- **License**: MIT OR Apache-2.0
- **Repository**: https://github.com/seanmonstar/reqwest

### axum
- **Purpose**: High-performance HTTP server framework for API endpoints
- **License**: MIT
- **Repository**: https://github.com/tokio-rs/axum

### clap
- **Purpose**: Command-line argument parsing
- **License**: MIT OR Apache-2.0
- **Repository**: https://github.com/clap-rs/clap

### comfy-table
- **Purpose**: Beautiful table formatting for terminal output
- **License**: MIT
- **Repository**: https://github.com/nukesor/comfy-table

## Full Dependency Tree

For a complete list of all dependencies and their licenses, run:

```bash
cargo tree --package policycheck
cargo install cargo-license && cargo license
```

## Standards Implemented

### Robots Exclusion Protocol (REP)
- **Specification**: [RFC 9309](https://www.rfc-editor.org/rfc/rfc9309.html)
- **Original Spec**: [robotstxt.org](https://www.robotstxt.org/)

### Responsible Sourcing License (RSL)
- **Specification**: [RSL Standard](https://rslstandard.org/rsl)
- **Section**: [Associating RSL Licenses with Digital Assets](https://rslstandard.org/rsl#_4-associating-rsl-licenses-with-digital-assets)

### Text and Data Mining Reservation Protocol (TDMRep)
- **Specification**: [W3C TDMRep Community Group](https://www.w3.org/community/tdmrep/)
- **Purpose**: Domain-wide TDM policies for AI training and research

### .well-known URIs
- **Registry**: [IANA Well-Known URIs](https://www.iana.org/assignments/well-known-uris/well-known-uris.xhtml)
- **Specification**: [RFC 8615](https://www.rfc-editor.org/rfc/rfc8615.html)

## Contributing

PolicyCheck is part of the [OpenAttribution](https://openattribution.org) initiative, dedicated to making web attribution transparent and accessible.

If you'd like to contribute:
- Report issues: [GitHub Issues](https://github.com/openattribution-org/policycheck/issues)
- Submit PRs: [GitHub Pull Requests](https://github.com/openattribution-org/policycheck/pulls)
- Join discussions: [GitHub Discussions](https://github.com/openattribution-org/policycheck/discussions)

## License

PolicyCheck is licensed under the MIT License. See [LICENSE](LICENSE) for details.

Third-party software notices are in [NOTICE](NOTICE).
