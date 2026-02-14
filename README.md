# 🔍 PolicyCheck

**Web Attribution and Compliance Scanner**

A fast, portable tool for checking web scraping compliance across robots.txt, RSL licenses, and TDM policies. Built with Rust for the [OpenAttribution](https://openattribution.org) initiative.

[![Crates.io](https://img.shields.io/crates/v/policycheck.svg)](https://crates.io/crates/policycheck)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![OpenAttribution](https://img.shields.io/badge/OpenAttribution-🔍-green.svg)](https://openattribution.org)

## What is PolicyCheck?

PolicyCheck helps you **scrape responsibly** by checking multiple compliance signals:

- ✅ **Robots.txt** - What paths you can crawl (REP/RFC 9309)
- 📜 **RSL Licenses** - Required licensing terms (Responsible Sourcing License)
- 🎯 **Content Signals** - AI usage preferences (Cloudflare's policy framework)
- 🤖 **TDM Policies** - Text & Data Mining permissions (coming soon)
- 🔒 **Privacy Controls** - DNT, GPC signals (coming soon)
- 📧 **Security Contacts** - Who to contact about scraping (coming soon)

## Features

- 🤖 **AI Bot Analysis** - Check 26 known AI crawlers (GPTBot, ClaudeBot, CCBot, etc.)
- 🎯 **Content Signals** - Detect Cloudflare's AI policy signals (search, ai-input, ai-train)
- 📊 **CSV Export** - Major AI bots as columns for advertiser analysis
- 🚀 **Fast** - Built with Rust, battle-tested parser (34M+ robots.txt files)
- 📦 **Portable** - Single binary, no dependencies
- 🔍 **Comprehensive** - User agents, crawl delays, sitemaps, paths, licenses
- 📜 **RSL License Detection** - Automatically finds Responsible Sourcing Licenses
- 📈 **Multiple Formats** - Table, JSON, CSV, or compact text output
- 🌐 **HTTP API** - Run as a service for integration
- 📝 **CSV Batch Processing** - Analyze thousands of URLs concurrently
- ⚡ **Concurrent** - Parallel URL analysis

## Web UI

Try PolicyCheck instantly at **[openattribution.org/policycheck](https://openattribution.org/policycheck/)**

- 🌐 No installation required
- 📊 Interactive analysis with visual results
- 📥 Export to CSV for bulk analysis
- 🤖 See AI bot blocking status at a glance

Perfect for quick checks before integrating the API or CLI.

## Quick Start

### Installation

#### From crates.io (Recommended)

Requires Rust 1.75+:

```bash
cargo install policycheck
```

#### From Source

For development or the latest unreleased features:

```bash
git clone https://github.com/openattribution-org/policycheck.git
cd policycheck
cargo build --release
```

The binary will be at `target/release/policycheck`.

### Basic Usage

```bash
# Analyze a single URL
policycheck analyze --url https://www.nytimes.com

# Check multiple URLs
policycheck analyze \
  --url https://www.nytimes.com \
  --url https://github.com \
  --url https://techcrunch.com

# Analyze from CSV file (advertiser use case)
policycheck analyze --csv publishers.csv --format csv --output results.csv

# Check for specific user agent
policycheck analyze --url https://www.nytimes.com --user-agent GPTBot

# Output as JSON
policycheck analyze --url https://www.nytimes.com --format json

# Output as CSV with AI bot columns
policycheck analyze --url https://www.nytimes.com --format csv

# Save to file
policycheck analyze --url https://www.nytimes.com --output results.json
```

## AI Bot Analysis

PolicyCheck analyzes **26 known AI crawlers** including GPTBot, ClaudeBot, CCBot, and more. Perfect for two key use cases:

### Publisher Use Case: Protecting Content

Check which AI training bots can access your content:

```bash
policycheck analyze --url https://www.nytimes.com --format compact
```

Shows comprehensive breakdown of which bots are blocked vs allowed.

### Advertiser Use Case: Evaluating Publisher Partnerships

Analyze multiple publishers to see which ones block AI search engines (affecting brand visibility):

```bash
policycheck analyze --csv publishers.csv --format csv --output analysis.csv
```

**Example output:**
```csv
URL,Status,Path Allowed,RSL Licenses,TDM Reserved,GPTBot,ClaudeBot,Google-Extended,Meta-ExternalAgent,CCBot,Bytespider,OAI-SearchBot,PerplexityBot
https://www.nytimes.com,Success,Yes,0,N/A,Blocked,Blocked,Blocked,Blocked,Blocked,Blocked,Blocked,Blocked
https://github.com,Success,Yes,0,N/A,Allowed,Allowed,Allowed,Allowed,Allowed,Allowed,Allowed,Allowed
https://techcrunch.com,Success,Yes,0,N/A,Blocked,Blocked,Blocked,Allowed,Blocked,Blocked,Allowed,Allowed
```

**Key insights:**
- **NYTimes**: Blocks all AI bots (zero AI search visibility)
- **GitHub**: Allows all AI bots (maximum AI discoverability)
- **TechCrunch**: Selectively blocks training bots, allows some search bots

Perfect for advertisers evaluating whether publisher placements will appear in ChatGPT, Perplexity, Claude, etc.

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
License: https://acme.com/global-license.xml

User-agent: *
Disallow: /private/
Allow: /public/

User-agent: GPTBot
Disallow: /
License: https://acme.com/gptbot-specific-license.xml
```

In this example:
- Most bots will see the global license
- GPTBot will see only the group-scoped license (global is ignored)

**Real-world example:** NYTimes blocks AI bots comprehensively:
```bash
policycheck analyze --url https://www.nytimes.com --user-agent GPTBot
# Shows: Blocked, with legal notice about prohibited uses
```

### RSL in Output

PolicyCheck reports three license fields:

- **`active_licenses`**: The licenses that actually apply (follows RSL precedence rules)
- **`global_licenses`**: Licenses defined outside user-agent groups
- **`group_licenses`**: Licenses defined for the specific user agent

**Compact output example:**
```
================================================================================
URL: https://www.nytimes.com
Robots.txt: https://www.nytimes.com/robots.txt
Status: ✓ Success

User Agents:
  • *
  • GPTBot
  • ClaudeBot
  • (40+ more...)

Path Access (for GPTBot): ✗ Disallowed

AI Bot Analysis:
  🚫 GPTBot: Blocked
  🚫 ClaudeBot: Blocked
  🚫 CCBot: Blocked
  ✓ Googlebot: Allowed (with restrictions)

Sitemaps:
  • https://www.nytimes.com/sitemaps/new/news.xml.gz
  • (15+ more sitemaps)
================================================================================
```

**JSON output example:**
```json
{
  "url": "https://github.com",
  "robots_url": "https://github.com/robots.txt",
  "status": "success",
  "user_agents": ["*"],
  "ai_bot_analysis": [
    {"bot_name": "GPTBot", "company": "OpenAI", "category": "Training", "status": "allowed"},
    {"bot_name": "ClaudeBot", "company": "Anthropic", "category": "Training", "status": "allowed"}
  ],
  "global_licenses": [],
  "group_licenses": [],
  "active_licenses": [],
  "crawl_delay": null,
  "sitemaps": ["https://github.com/sitemap.xml"],
  "is_path_allowed": true
}
```

For more information about RSL, see the [RSL Standard](https://rslstandard.org/rsl#_4-associating-rsl-licenses-with-digital-assets).

## Content Signals (Cloudflare AI Policy Framework)

PolicyCheck automatically detects **Content Signals** - Cloudflare's framework for expressing AI usage preferences in robots.txt. Adopted by over 3.8 million domains using Cloudflare's managed robots.txt.

### What are Content Signals?

Content Signals allow websites to express preferences for how their content can be used **after** it's been accessed. Three signals are defined:

- **`search`** - Traditional search indexing and results (not AI-generated summaries)
- **`ai-input`** - Inputting content into AI models (RAG, grounding, generative AI search)
- **`ai-train`** - Training or fine-tuning AI models

### Format

```
User-agent: *
Content-Signal: search=yes, ai-train=no, ai-input=yes
Allow: /
```

Values can be `yes` (permitted) or `no` (not permitted). Omitting a signal means no preference is expressed.

### Example Output

**Compact format:**
```
Content Signals:
  ✓ search: yes
  ✗ ai-train: no
  ✓ ai-input: yes
```

**CSV format** includes columns: `CS-Search`, `CS-AI-Input`, `CS-AI-Train`

**JSON format:**
```json
{
  "content_signal_search": "yes",
  "content_signal_ai_input": "yes",
  "content_signal_ai_train": "no"
}
```

### Real-World Example

```bash
policycheck analyze --url https://blog.cloudflare.com --format compact
```

Cloudflare's blog permits all AI usage:
- `search=yes` - Allowed in search indexes
- `ai-input=yes` - Allowed for AI search/RAG
- `ai-train=yes` - Allowed for model training

For more information, see [Cloudflare's Content Signals announcement](https://blog.cloudflare.com/content-signals-policy).

## Output Formats

### CSV Format (Best for Advertisers)

**Perfect for bulk analysis with AI bot columns:**

```bash
policycheck analyze --csv publishers.csv --format csv --output analysis.csv
```

Creates a spreadsheet with major AI bots as columns - ideal for Excel/Google Sheets analysis:

```csv
URL,Status,Path Allowed,RSL Licenses,TDM Reserved,GPTBot,ClaudeBot,Google-Extended,Meta-ExternalAgent,CCBot,Bytespider,OAI-SearchBot,PerplexityBot
https://www.nytimes.com,Success,Yes,0,N/A,Blocked,Blocked,Blocked,Blocked,Blocked,Blocked,Blocked,Blocked
https://github.com,Success,Yes,0,N/A,Allowed,Allowed,Allowed,Allowed,Allowed,Allowed,Allowed,Allowed
```

### Table Format (Default)

Perfect for quick checks:

```bash
policycheck analyze --url https://github.com --format table
```

Shows summary information in a clean ASCII table.

### Compact Format

Detailed, human-readable output with full AI bot breakdown:

```bash
policycheck analyze --url https://www.nytimes.com --format compact
```

Shows all details including blocked/allowed AI bots, paths, sitemaps, and licenses.

### JSON Format

For programmatic use:

```bash
policycheck analyze --url https://www.nytimes.com --format json > results.json
```

Includes `ai_bot_analysis` array with per-bot status - perfect for integration with other tools.

## Running as a Service

### Production API

The PolicyCheck API is available at **https://policycheck-d7wv0g.fly.dev**

No authentication required for public use. Rate limits may apply.

### Run Your Own Server

Start the HTTP API server locally:

```bash
policycheck serve --port 3000 --host 0.0.0.0
```

**Features:**
- ✅ CORS enabled (all origins)
- ✅ JSON request/response
- ✅ Concurrent request handling
- ✅ 10s timeout per URL

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
  "urls": ["https://www.nytimes.com", "https://github.com"],
  "user_agent": "GPTBot"
}
```

**Success Response:**
```json
{
  "total": 2,
  "successful": 2,
  "failed": 0,
  "results": [
    {
      "url": "https://www.nytimes.com",
      "robots_url": "https://www.nytimes.com/robots.txt",
      "status": "success",
      "user_agents": ["*", "GPTBot", "ClaudeBot", "..."],
      "crawl_delay": null,
      "sitemaps": ["https://www.nytimes.com/sitemaps/new/news.xml.gz"],
      "allowed_paths": [],
      "disallowed_paths": ["/"],
      "is_path_allowed": false,
      "global_licenses": [],
      "group_licenses": [],
      "active_licenses": [],
      "ai_bot_analysis": [
        {"bot_name": "GPTBot", "company": "OpenAI", "category": "Training", "status": "blocked"},
        {"bot_name": "ClaudeBot", "company": "Anthropic", "category": "Training", "status": "blocked"}
      ],
      "error": null
    }
  ]
}
```

**Error Response (with failures):**
```json
{
  "total": 2,
  "successful": 1,
  "failed": 1,
  "results": [
    {
      "url": "https://invalid-domain-xyz.com",
      "robots_url": "https://invalid-domain-xyz.com/robots.txt",
      "status": "fetch_error",
      "error": "Failed to fetch robots.txt",
      "user_agents": [],
      "ai_bot_analysis": []
    },
    {
      "url": "https://github.com",
      "status": "success",
      "error": null
    }
  ]
}
```

### Example with curl

**Using production API:**
```bash
curl -X POST https://policycheck-d7wv0g.fly.dev/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://www.nytimes.com"],
    "user_agent": "GPTBot"
  }'
```

**Using local server:**
```bash
curl -X POST http://localhost:3000/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://www.nytimes.com"],
    "user_agent": "GPTBot"
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

def check_ai_bot_access(urls, user_agent="GPTBot"):
    response = requests.post(
        "http://localhost:3000/analyze",
        json={"urls": urls, "user_agent": user_agent}
    )
    return response.json()

# Advertiser use case: check which publishers block AI bots
publishers = [
    "https://www.nytimes.com",
    "https://github.com",
    "https://techcrunch.com"
]

result = check_ai_bot_access(publishers)
for site in result['results']:
    print(f"\n{site['url']}")
    print(f"  GPTBot access: {'❌ Blocked' if not site['is_path_allowed'] else '✅ Allowed'}")

    # Check specific AI bots
    for bot in site['ai_bot_analysis']:
        if bot['bot_name'] in ['GPTBot', 'OAI-SearchBot', 'PerplexityBot']:
            status = '❌' if bot['status'] == 'blocked' else '✅'
            print(f"  {status} {bot['bot_name']}")
```

### Node.js

```javascript
const axios = require('axios');

async function checkAIBotAccess(urls, userAgent = 'GPTBot') {
  const response = await axios.post('http://localhost:3000/analyze', {
    urls,
    user_agent: userAgent
  });
  return response.data;
}

// Advertiser use case: analyze publisher AI visibility
const publishers = [
  'https://www.nytimes.com',
  'https://github.com',
  'https://techcrunch.com'
];

const result = await checkAIBotAccess(publishers);
console.log(`Analyzed ${result.total} publishers`);

result.results.forEach(site => {
  const blocked = site.ai_bot_analysis.filter(b => b.status === 'blocked').length;
  const allowed = site.ai_bot_analysis.filter(b => b.status === 'allowed').length;
  console.log(`${site.url}: ${blocked} blocked, ${allowed} allowed`);
});
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
- [ ] GitHub Action for PR compliance checks
- [ ] Pre-commit hook for URL validation

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
- ✅ **Content Signals**: Cloudflare's AI Policy Framework (CC0 License)
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

Third-party software notices are in [NOTICE](NOTICE).

### Key Dependencies

- **texting_robots** (MIT OR Apache-2.0) - Robust robots.txt parsing by [@Smerity](https://github.com/Smerity)
- **reqwest** (MIT OR Apache-2.0) - HTTP client
- **clap** (MIT OR Apache-2.0) - CLI argument parsing
- **axum** (MIT) - HTTP server framework
- See [NOTICE](NOTICE) for complete attribution list

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
