"""Google Gemini 3 Flash provider with google_search grounding."""

from __future__ import annotations

import os

import requests
from google import genai
from google.genai import types

from . import Citation, SearchResult


class GeminiProvider:
    name = "gemini"
    model = "gemini-3-flash-preview"
    crawler_bot = "Google-Extended"

    def __init__(self) -> None:
        self.client = genai.Client(api_key=os.environ["GEMINI_API_KEY"])

    def search(self, prompt: str) -> SearchResult:
        response = self.client.models.generate_content(
            model=self.model,
            contents=prompt,
            config=types.GenerateContentConfig(
                tools=[types.Tool(google_search=types.GoogleSearch())]
            ),
        )

        text = response.text or ""
        citations: list[Citation] = []

        metadata = getattr(response.candidates[0], "grounding_metadata", None)
        if metadata:
            chunks = getattr(metadata, "grounding_chunks", []) or []
            for rank, chunk in enumerate(chunks, 1):
                web = getattr(chunk, "web", None)
                if not web:
                    continue
                uri = getattr(web, "uri", "") or ""
                title = getattr(web, "title", "") or ""
                url = self._resolve_redirect(uri) if uri else uri
                if url:
                    citations.append(Citation.from_url(url, title, rank))

        return SearchResult(
            text=text,
            citations=citations,
            model=self.model,
            provider=self.name,
        )

    @staticmethod
    def _resolve_redirect(url: str) -> str:
        """Resolve Google redirect URLs to their final destination."""
        try:
            resp = requests.head(url, allow_redirects=True, timeout=5)
            return resp.url
        except requests.RequestException:
            return url
