#!/usr/bin/env python3
"""
Example Python client for policycheck HTTP API
"""

import requests
import json


class PolicyCheckClient:
    """Client for interacting with policycheck HTTP API"""

    def __init__(self, base_url="http://localhost:3000"):
        self.base_url = base_url.rstrip("/")

    def health_check(self):
        """Check if the service is healthy"""
        response = requests.get(f"{self.base_url}/health")
        return response.json()

    def analyze(self, urls, user_agent="*"):
        """
        Analyze robots.txt for given URLs

        Args:
            urls: List of URLs to analyze
            user_agent: User agent string to check permissions for

        Returns:
            dict with analysis results
        """
        response = requests.post(
            f"{self.base_url}/analyze",
            json={"urls": urls, "user_agent": user_agent},
        )
        response.raise_for_status()
        return response.json()


def main():
    # Create client
    client = PolicyCheckClient()

    # Check health
    print("Health check:", client.health_check())
    print()

    # Analyze some URLs
    urls = [
        "https://github.com",
        "https://reddit.com",
        "https://stackoverflow.com",
    ]

    print(f"Analyzing {len(urls)} URLs...")
    result = client.analyze(urls, user_agent="MyBot")

    print(f"\nResults:")
    print(f"  Total: {result['total']}")
    print(f"  Successful: {result['successful']}")
    print(f"  Failed: {result['failed']}")

    # Print details for each site
    for site in result["results"]:
        print(f"\n{site['url']}:")
        print(f"  Status: {site['status']}")

        if site["status"] == "success":
            print(f"  Path Allowed: {site['is_path_allowed']}")
            print(f"  User Agents: {', '.join(site['user_agents'][:3])}...")
            print(f"  Crawl Delay: {site.get('crawl_delay', 'None')}")
            print(f"  Disallowed Paths: {len(site['disallowed_paths'])}")
            print(f"  Sitemaps: {len(site['sitemaps'])}")
        else:
            print(f"  Error: {site.get('error', 'Unknown error')}")


if __name__ == "__main__":
    main()
