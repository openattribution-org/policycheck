"""OpenAI provider with built-in web_search tool."""

from __future__ import annotations

from openai import OpenAI

from . import Citation, SearchResult


class OpenAIProvider:
    name = "openai"
    model = "gpt-5-mini"
    crawler_bot = "GPTBot"

    def __init__(self) -> None:
        self.client = OpenAI()

    def search(self, prompt: str) -> SearchResult:
        response = self.client.responses.create(
            model=self.model,
            tools=[{"type": "web_search"}],
            input=prompt,
        )

        text_parts: list[str] = []
        citations: list[Citation] = []
        seen_urls: set[str] = set()
        rank = 0

        for item in response.output:
            if item.type != "message":
                continue
            for block in item.content:
                if block.type == "output_text":
                    text_parts.append(block.text)
                    for ann in getattr(block, "annotations", []):
                        if ann.type == "url_citation" and ann.url not in seen_urls:
                            seen_urls.add(ann.url)
                            rank += 1
                            citations.append(
                                Citation.from_url(ann.url, ann.title, rank)
                            )

        return SearchResult(
            text="\n".join(text_parts),
            citations=citations,
            model=self.model,
            provider=self.name,
        )
