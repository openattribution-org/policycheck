# 🔍 PolicyCheck

**Web Attribution and Compliance Scanner**

A fast, portable tool for checking web scraping compliance across robots.txt, RSL licenses, and TDM policies. Built with Rust for the [OpenAttribution](https://openattribution.org) initiative.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![OpenAttribution](https://img.shields.io/badge/OpenAttribution-🔍-green.svg)](https://openattribution.org)

## What is PolicyCheck?

PolicyCheck helps you **scrape responsibly** by checking multiple compliance signals:

- ✅ **Robots.txt** - What paths you can crawl (REP/RFC 9309)
- 📜 **RSL Licenses** - Required licensing terms (Responsible Sourcing License)
- 🤖 **TDM Policies** - Text & Data Mining permissions (coming soon)
- 🔒 **Privacy Controls** - DNT, GPC signals (coming soon)
- 📧 **Security Contacts** - Who to contact about scraping (coming soon)

## Features

- 🚀 **Fast** - Built with Rust, battle-tested parser (34M+ robots.txt files)
- 📦 **Portable** - Single binary, no dependencies
- 🔍 **Comprehensive** - User agents, crawl delays, sitemaps, paths, licenses
- 📜 **RSL License Detection** - Automatically finds Responsible Sourcing Licenses
- 📊 **Multiple Formats** - Table, JSON, or compact text output
- 🌐 **HTTP API** - Run as a service for integration
- 📝 **CSV Batch Processing** - Analyze thousands of URLs
- ⚡ **Concurrent** - Parallel URL analysis

## Quick Start

### Installation

#### From Source (Recommended)

Requires Rust 1.85+ (for edition 2024 support):

```bash
git clone https://github.com/openattribution-org/policycheck.git
cd policycheck
cargo build --release
```

The binary will be at `target/release/policycheck`.

#### Using Cargo

```bash
cargo install --path .
```

### Basic Usage

```bash
# Analyze a single URL
policycheck analyze --url https://example.com

# Check multiple URLs
policycheck analyze \
  --url https://github.com \
  --url https://openai.com \
  --url https://anthropic.com

# Analyze from CSV file
policycheck analyze --csv urls.csv

# Check for specific user agent
policycheck analyze --url https://example.com --user-agent GPTBot

# Output as JSON
policycheck analyze --url https://example.com --format json

# Save to file
policycheck analyze --url https://example.com --output results.json
```

## RSL (Responsible Sourcing License) Support

PolicyCheck automatically detects **RSL license directives** from robots.txt files. RSL extends the Robots Exclusion Protocol to enable websites to declare governing license documents for automated crawlers.

### How RSL Works

RSL introduces a `License:` directive that can be:
- **Global**: Outside any User-agent group (applies to all bots)
- **Group-scoped**: Inside a User-agent group (applies only to that bot)

**Precedence rule**: Group-scoped licenses override global licenses.

### Example robots.txt with RSL

```
# Global license (applies to all bots unless overridden)
License: https://example.com/global-license.xml

User-agent: *
Disallow: /private/
Allow: /public/

User-agent: GPTBot
Disallow: /
License: https://example.com/gptbot-specific-license.xml
```

In this example:
- Most bots will see the global license
- GPTBot will see only the group-scoped license (global is ignored)

### RSL in Output

PolicyCheck reports three license fields:

- **`active_licenses`**: The licenses that actually apply (follows RSL precedence rules)
- **`global_licenses`**: Licenses defined outside user-agent groups
- **`group_licenses`**: Licenses defined for the specific user agent

**Compact output example:**
```
================================================================================
URL: https://example.com
Robots.txt: https://example.com/robots.txt
Status: ✓ Success

User Agents:
  • *
  • GPTBot

Path Access: ✓ Allowed

RSL Licenses (Active):
  📜 https://example.com/gptbot-specific-license.xml

Sitemaps:
  • https://example.com/sitemap.xml
================================================================================
```

**JSON output example:**
```json
{
  "url": "https://example.com",
  "robots_url": "https://example.com/robots.txt",
  "status": "success",
  "user_agents": ["*", "GPTBot"],
  "global_licenses": ["https://example.com/global-license.xml"],
  "group_licenses": ["https://example.com/gptbot-specific-license.xml"],
  "active_licenses": ["https://example.com/gptbot-specific-license.xml"],
  "crawl_delay": null,
  "sitemaps": ["https://example.com/sitemap.xml"],
  "is_path_allowed": true
}
```

For more information about RSL, see the [RSL Standard](https://rslstandard.org/rsl#_4-associating-rsl-licenses-with-digital-assets).

## Output Formats

### Table Format (Default)

Perfect for quick checks across multiple sites:

```bash
policycheck analyze --url https://github.com --format table
```

```
╭──────────────────┬──────────┬─────────────┬─────────────┬──────────────┬──────────────┬──────────┬────────────╮
│ URL              │ Status   │ User Agents │ Crawl Delay │ Path Allowed │ RSL Licenses │ Sitemaps │ Disallowed │
├──────────────────┼──────────┼─────────────┼─────────────┼──────────────┼──────────────┼──────────┼────────────┤
│ https://github.… │ ✓ Success│ *, GoogleBot│ -           │ ✓ Yes        │ 2            │ 1        │ 45         │
╰──────────────────┴──────────┴─────────────┴─────────────┴──────────────┴──────────────┴──────────┴────────────╯
```

### Compact Format

Detailed, human-readable output:

```bash
policycheck analyze --url https://example.com --format compact
```

Shows all details including full paths, sitemaps, and license URLs.

### JSON Format

For programmatic use:

```bash
policycheck analyze --url https://example.com --format json > results.json
```

Perfect for integration with other tools.

## Running as a Service

Start the HTTP API server:

```bash
policycheck serve --port 3000 --host 0.0.0.0
```

### API Endpoints

#### `GET /health`

Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "service": "policycheck",
  "version": "0.1.0"
}
```

#### `POST /analyze`

Analyze robots.txt and RSL licenses for given URLs.

**Request:**
```json
{
  "urls": ["https://example.com", "https://github.com"],
  "user_agent": "MyBot/1.0"
}
```

**Response:**
```json
{
  "total": 2,
  "successful": 2,
  "failed": 0,
  "results": [
    {
      "url": "https://example.com",
      "robots_url": "https://example.com/robots.txt",
      "status": "success",
      "user_agents": ["*", "Googlebot"],
      "crawl_delay": 1.0,
      "sitemaps": ["https://example.com/sitemap.xml"],
      "allowed_paths": ["/public"],
      "disallowed_paths": ["/private"],
      "is_path_allowed": true,
      "global_licenses": ["https://example.com/license.xml"],
      "group_licenses": [],
      "active_licenses": ["https://example.com/license.xml"],
      "error": null
    }
  ]
}
```

### Example with curl

```bash
curl -X POST http://localhost:3000/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "user_agent": "MyBot"
  }'
```

## CSV Batch Processing

Create a CSV file with URLs to check:

```csv
url
https://acme.com
https://example.org
https://test.io
```

Or with identifiers for tracking:

```csv
source_id,url
acme,https://acme.com
example,https://example.org
test,https://test.io
```

Analyze all URLs:

```bash
policycheck analyze --csv partners.csv --format compact > results.txt
```

PolicyCheck will automatically:
- Find the URL column (looks for headers containing "url", "link", "website", etc.)
- Default to the first column if no URL header is found
- Add `https://` prefix if missing
- Skip empty rows
- Process all URLs in parallel

**Note**: Only the URL column is used for analysis. Additional columns (like `source_id`) can be present for your own tracking but are ignored by PolicyCheck.

## Integration Examples

### Python

```python
import requests

def check_compliance(urls, user_agent="MyBot/1.0"):
    response = requests.post(
        "http://localhost:3000/analyze",
        json={"urls": urls, "user_agent": user_agent}
    )
    return response.json()

# Usage
result = check_compliance(["https://example.com"])
for site in result['results']:
    print(f"\n{site['url']}")
    print(f"  Allowed: {site['is_path_allowed']}")
    if site['active_licenses']:
        print(f"  Licenses: {', '.join(site['active_licenses'])}")
```

### Node.js

```javascript
const axios = require('axios');

async function checkCompliance(urls, userAgent = 'MyBot/1.0') {
  const response = await axios.post('http://localhost:3000/analyze', {
    urls,
    user_agent: userAgent
  });
  return response.data;
}

// Usage
const result = await checkCompliance(['https://example.com']);
console.log(`Checked ${result.total} URLs, ${result.successful} successful`);
```

### Go

```go
package main

import (
    "bytes"
    "encoding/json"
    "net/http"
)

type AnalyzeRequest struct {
    URLs      []string `json:"urls"`
    UserAgent string   `json:"user_agent"`
}

func checkCompliance(urls []string, userAgent string) (*AnalyzeResponse, error) {
    reqBody := AnalyzeRequest{URLs: urls, UserAgent: userAgent}
    jsonData, _ := json.Marshal(reqBody)

    resp, err := http.Post(
        "http://localhost:3000/analyze",
        "application/json",
        bytes.NewBuffer(jsonData),
    )
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    var result AnalyzeResponse
    json.NewDecoder(resp.Body).Decode(&result)
    return &result, nil
}
```

## Deployment

### Docker

```dockerfile
FROM rust:1.92-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/policycheck /usr/local/bin/policycheck
EXPOSE 3000
CMD ["policycheck", "serve", "--host", "0.0.0.0", "--port", "3000"]
```

Build and run:
```bash
docker build -t policycheck .
docker run -p 3000:3000 policycheck
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: policycheck
spec:
  replicas: 3
  selector:
    matchLabels:
      app: policycheck
  template:
    metadata:
      labels:
        app: policycheck
    spec:
      containers:
      - name: policycheck
        image: openattribution/policycheck:latest
        ports:
        - containerPort: 3000
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
---
apiVersion: v1
kind: Service
metadata:
  name: policycheck-service
spec:
  selector:
    app: policycheck
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
```

## Roadmap

### ✅ Completed
- [x] Robots.txt parsing (REP/RFC 9309)
- [x] RSL license detection
- [x] User agent matching
- [x] Crawl delay detection
- [x] Sitemap discovery
- [x] Path permission checking
- [x] CSV batch processing
- [x] HTTP API server
- [x] Multiple output formats

### 🚧 In Progress
- [ ] TDM (Text & Data Mining) policy detection (`/.well-known/tdmrep.json`)
- [ ] Security contact discovery (`/.well-known/security.txt`)
- [ ] Privacy control detection (DNT, GPC)

### 📋 Planned
- [ ] AI plugin manifest detection (`/.well-known/ai-plugin.json`)
- [ ] OpenID configuration for gated content
- [ ] Caching layer for repeated checks
- [ ] Web UI dashboard
- [ ] GitHub Action for PR compliance checks
- [ ] Pre-commit hook for URL validation

See [WELL_KNOWN_PROPOSAL.md](WELL_KNOWN_PROPOSAL.md) for detailed implementation plans.

## Command Reference

### `policycheck analyze`

Analyze robots.txt and RSL licenses from URLs.

**Options:**
- `-u, --url <URL>` - URL to analyze (can be repeated)
- `-c, --csv <PATH>` - CSV file containing URLs
- `-a, --user-agent <AGENT>` - User agent to check (default: "*")
- `-f, --format <FORMAT>` - Output format: table, json, compact (default: table)
- `-o, --output <PATH>` - Save output to file

### `policycheck serve`

Start HTTP API server.

**Options:**
- `-p, --port <PORT>` - Port to listen on (default: 3000)
- `--host <HOST>` - Host to bind to (default: 127.0.0.1)

## Performance

PolicyCheck is designed for speed:

- **Concurrent analysis**: Multiple URLs analyzed in parallel
- **Optimized builds**: Release builds use LTO and aggressive optimization
- **Battle-tested parser**: Based on `texting_robots`, tested against 34M+ real-world files
- **Low memory footprint**: Efficient parsing with minimal allocations

Typical performance:
- Single URL analysis: ~50-200ms (network dependent)
- 100 URLs analyzed concurrently: ~2-5 seconds

## Security Considerations

- **Input validation**: URLs are validated before processing
- **Size limits**: robots.txt files limited to 500KB (Google's recommendation)
- **Timeouts**: HTTP requests timeout after 10 seconds
- **No arbitrary code execution**: Pure parsing, no eval or dynamic code
- **CORS enabled**: API server has CORS enabled by default

## Standards Compliance

PolicyCheck implements the following standards:

- ✅ **RFC 9309**: Robots Exclusion Protocol (REP)
- ✅ **RSL Standard**: Responsible Sourcing License
- 🚧 **W3C TDMRep**: Text and Data Mining Reservation Protocol (planned)
- 🚧 **RFC 9116**: security.txt (planned)
- 🚧 **RFC 8615**: Well-Known URIs (planned)

## Contributing

PolicyCheck is part of the [OpenAttribution](https://openattribution.org) initiative. Contributions welcome!

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

Third-party software notices and attributions are in [NOTICE](NOTICE) and [ATTRIBUTIONS.md](ATTRIBUTIONS.md).

### Key Dependencies

- **texting_robots** (MIT OR Apache-2.0) - Robust robots.txt parsing by [@Smerity](https://github.com/Smerity)
- See [ATTRIBUTIONS.md](ATTRIBUTIONS.md) for complete list

## OpenAttribution Initiative

PolicyCheck is built for the [OpenAttribution](https://openattribution.org) initiative, which aims to make web attribution transparent, accessible, and machine-readable.

**Mission**: Enable responsible AI development through clear content licensing and attribution standards.

## Support

- 🐛 **Report issues**: [GitHub Issues](https://github.com/openattribution-org/policycheck/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/openattribution-org/policycheck/discussions)
- 📧 **Contact**: [openattribution.org](https://openattribution.org)
- 🌐 **Website**: [OpenAttribution.org](https://openattribution.org)

## Acknowledgments

Built with ❤️ by the OpenAttribution community.

Special thanks to:
- [@Smerity](https://github.com/Smerity) for texting_robots
- The Rust community for excellent tooling
- Everyone contributing to open web standards

---

**Made with Rust 🦀 | Part of OpenAttribution 🔍 | MIT Licensed 📜**
