# 🚀 robotxt Quick Start Guide

## Installation

### Option 1: Use the binary directly
```bash
# The optimized binary is ready to use:
./target/release/robotxt --help

# Or copy it to your PATH:
sudo cp target/release/robotxt /usr/local/bin/
```

### Option 2: Install with cargo
```bash
cargo install --path .
```

## Basic Examples

### 1. Analyze a single website
```bash
robotxt analyze --url https://github.com
```

### 2. Analyze multiple websites
```bash
robotxt analyze \
  --url https://github.com \
  --url https://reddit.com \
  --url https://stackoverflow.com
```

### 3. Analyze from CSV file
```bash
# Use the included example.csv or create your own:
robotxt analyze --csv example.csv
```

### 4. Check for a specific user agent
```bash
robotxt analyze \
  --url https://github.com \
  --user-agent "Googlebot"
```

### 5. Get JSON output
```bash
robotxt analyze \
  --url https://github.com \
  --format json
```

### 6. Save results to file
```bash
robotxt analyze \
  --csv example.csv \
  --format json \
  --output analysis.json
```

## Running as a Service

### Start the server
```bash
robotxt serve --port 3000 --host 0.0.0.0
```

### Test the API
```bash
# Health check
curl http://localhost:3000/health

# Analyze URLs
curl -X POST http://localhost:3000/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://github.com", "https://reddit.com"],
    "user_agent": "MyBot"
  }'
```

## Output Formats

### Table (default)
Best for quick visual inspection:
```bash
robotxt analyze --url https://github.com
```

### JSON
Best for programmatic use:
```bash
robotxt analyze --url https://github.com --format json
```

### Compact
Best for detailed human-readable output:
```bash
robotxt analyze --url https://github.com --format compact
```

## CSV File Format

Create a file called `urls.csv`:
```csv
url
https://example.com
https://github.com
https://reddit.com
```

Then analyze:
```bash
robotxt analyze --csv urls.csv
```

## Common Use Cases

### Check if your bot can crawl specific paths
```bash
robotxt analyze \
  --url https://example.com/specific/path \
  --user-agent "MyBot"
```

Look for `Path Allowed` in the output.

### Find crawl delays
```bash
robotxt analyze \
  --csv sites.csv \
  --format compact | grep "Crawl Delay"
```

### Export analysis for multiple sites
```bash
robotxt analyze \
  --csv sites.csv \
  --format json \
  --output analysis.json
```

### Integrate with other tools
```bash
# Pipe to jq for JSON processing
robotxt analyze --url https://github.com --format json | jq '.[] | {url, crawl_delay}'

# Pipe to grep for filtering
robotxt analyze --csv sites.csv --format compact | grep -A 5 "Success"
```

## Performance Tips

1. **Use release build**: Always use `target/release/robotxt` in production
2. **Batch processing**: Process multiple URLs in one command for better performance
3. **JSON output**: Use JSON format when integrating with other systems
4. **Service mode**: Run as a service for high-volume analysis

## Troubleshooting

### "Fetch Error" messages
- Check your internet connection
- Verify the URL is correct
- Some sites may block automated requests
- Try adding a delay between requests

### "Parse Error" messages
- The robots.txt file may have invalid syntax
- This is informational - the tool continues processing

### Server won't start
- Check if port is already in use: `lsof -i :3000`
- Try a different port: `robotxt serve --port 8080`

## Next Steps

- Read the full [README.md](README.md) for deployment options
- Check the [examples/](examples/) directory for integration code
- Run `robotxt --help` for all available options
