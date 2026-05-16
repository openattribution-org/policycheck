"""Anthropic Claude provider with built-in web_search tool."""

from __future__ import annotations

from anthropic import Anthropic

from . import Citation, SearchResult


class AnthropicProvider:
    name = "anthropic"
    model = "claude-sonnet-4-6"
    crawler_bot = "ClaudeBot"

    def __init__(self) -> None:
        self.client = Anthropic()

    def search(self, prompt: str) -> SearchResult:
        message = self.client.messages.create(
            model=self.model,
            max_tokens=2048,
            messages=[{"role": "user", "content": prompt}],
            tools=[
                {
                    "type": "web_search_20250305",
                    "name": "web_search",
                    "max_uses": 5,
                }
            ],
        )

        text_parts: list[str] = []
        citations: list[Citation] = []
        seen_urls: set[str] = set()
        rank = 0

        for block in message.content:
            if getattr(block, "type", None) != "text":
                continue
            text_parts.append(getattr(block, "text", ""))
            for cite in getattr(block, "citations", None) or []:
                if getattr(cite, "type", None) != "web_search_result_location":
                    continue
                url = getattr(cite, "url", "")
                if not url or url in seen_urls:
                    continue
                seen_urls.add(url)
                rank += 1
                citations.append(
                    Citation.from_url(url, getattr(cite, "title", "") or "", rank)
                )

        return SearchResult(
            text="\n".join(text_parts),
            citations=citations,
            model=self.model,
            provider=self.name,
        )
