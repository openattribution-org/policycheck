"""Perplexity Sonar Pro provider with built-in web search."""

from __future__ import annotations

from perplexity import Perplexity

from . import Citation, SearchResult


class PerplexityProvider:
    name = "perplexity"
    model = "sonar"
    crawler_bot = "PerplexityBot"

    def __init__(self) -> None:
        self.client = Perplexity()

    def search(self, prompt: str) -> SearchResult:
        completion = self.client.chat.completions.create(
            model=self.model,
            messages=[{"role": "user", "content": prompt}],
        )

        text = completion.choices[0].message.content or ""
        citations: list[Citation] = []

        # Sonar returns citations as a list of URL strings
        raw_citations = getattr(completion, "citations", []) or []
        # search_results has richer data (title, url, snippet)
        search_results = getattr(completion, "search_results", []) or []

        # Build title lookup from search_results
        title_map: dict[str, str] = {}
        for sr in search_results:
            if isinstance(sr, dict):
                title_map[sr.get("url", "")] = sr.get("title", "")
            else:
                title_map[getattr(sr, "url", "")] = getattr(sr, "title", "")

        for rank, url in enumerate(raw_citations, 1):
            title = title_map.get(url, "")
            citations.append(Citation.from_url(url, title, rank))

        return SearchResult(
            text=text,
            citations=citations,
            model=self.model,
            provider=self.name,
        )
