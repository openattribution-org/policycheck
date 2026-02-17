"""Search provider protocol and shared data types for citation research."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol, runtime_checkable
from urllib.parse import urlparse


@dataclass
class Citation:
    """A single citation extracted from a search response."""

    url: str
    title: str
    domain: str
    rank: int  # 1-indexed position in citation list

    @classmethod
    def from_url(cls, url: str, title: str, rank: int) -> Citation:
        domain = urlparse(url).netloc.removeprefix("www.")
        return cls(url=url, title=title, domain=domain, rank=rank)


@dataclass
class SearchResult:
    """Result from a single provider search call."""

    text: str
    citations: list[Citation]
    model: str
    provider: str


@runtime_checkable
class SearchProvider(Protocol):
    """Protocol that all provider implementations must follow."""

    name: str  # "openai", "gemini", "perplexity"
    model: str  # "gpt-5", "gemini-3-flash", "sonar-pro"
    crawler_bot: str  # "GPTBot", "Google-Extended", "PerplexityBot"

    def search(self, prompt: str) -> SearchResult: ...
