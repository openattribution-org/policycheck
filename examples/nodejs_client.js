#!/usr/bin/env node
/**
 * Example Node.js client for policycheck HTTP API
 */

class PolicyCheckClient {
  constructor(baseUrl = 'http://localhost:3000') {
    this.baseUrl = baseUrl.replace(/\/$/, '');
  }

  async healthCheck() {
    const response = await fetch(`${this.baseUrl}/health`);
    return response.json();
  }

  async analyze(urls, userAgent = '*') {
    const response = await fetch(`${this.baseUrl}/analyze`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        urls,
        user_agent: userAgent
      })
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json();
  }
}

async function main() {
  // Create client
  const client = new PolicyCheckClient();

  // Check health
  const health = await client.healthCheck();
  console.log('Health check:', health);
  console.log();

  // Analyze some URLs
  const urls = [
    'https://github.com',
    'https://reddit.com',
    'https://stackoverflow.com'
  ];

  console.log(`Analyzing ${urls.length} URLs...`);
  const result = await client.analyze(urls, 'MyBot');

  console.log('\nResults:');
  console.log(`  Total: ${result.total}`);
  console.log(`  Successful: ${result.successful}`);
  console.log(`  Failed: ${result.failed}`);

  // Print details for each site
  for (const site of result.results) {
    console.log(`\n${site.url}:`);
    console.log(`  Status: ${site.status}`);

    if (site.status === 'success') {
      console.log(`  Path Allowed: ${site.is_path_allowed}`);
      console.log(`  User Agents: ${site.user_agents.slice(0, 3).join(', ')}...`);
      console.log(`  Crawl Delay: ${site.crawl_delay ?? 'None'}`);
      console.log(`  Disallowed Paths: ${site.disallowed_paths.length}`);
      console.log(`  Sitemaps: ${site.sitemaps.length}`);
    } else {
      console.log(`  Error: ${site.error ?? 'Unknown error'}`);
    }
  }
}

main().catch(console.error);
