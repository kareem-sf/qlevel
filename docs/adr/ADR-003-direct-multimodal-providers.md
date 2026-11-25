# ADR-003: Direct Cloud Multimodal Providers & Live Model Filtering

- **Status**: Accepted
- **Date**: 2026-11-25

## Context

Subcontractor quotations combine bilingual Arabic/English tables, scanned stamped PDFs, and multi-page technical exclusion clauses. Small local quantized models fail on dense Arabic OCR and table alignment, while third-party proxy aggregators introduce an extra data-custody hop for confidential tender pricing. Furthermore, provider APIs evolve rapidly:

- OpenAI supports dynamic OAuth PKCE token sharing for open-source local apps (`Sign in with ChatGPT`) with dynamic `client_id` assignment (`oaiapp_...`) and streaming `/v1/responses` ([OpenAI SIWC Overview](https://developers.openai.com/siwc/token-sharing-open-source), [SIWC Sign-In](https://developers.openai.com/siwc/token-sharing-open-source/sign-in), [SIWC Models & Inference](https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference)).
- Anthropic forbids third-party apps from spoofing personal `claude.ai` OAuth sessions ([Claude Legal & Compliance](https://code.claude.com/docs/en/legal-and-compliance)) and removed `temperature`/`top_p`/`top_k` in API version `2026-10-21` in favor of `thinking: {"type": "adaptive"}` ([Claude Migration 2026-10-21](https://platform.claude.com/docs/en/api/migration-2026-10-21)).
- Google Gemini `v1beta` deprecates `gemini-2.5-*` on Dec 1, 2026 in favor of `gemini-3.8-flash` and `gemini-4-*` ([Gemini Deprecations](https://aistudio.google.com/docs/deprecations), [Gemini Models](https://aistudio.google.com/docs/models)), and unpaid free-tier keys allow training on submitted data ([Gemini Pricing & Privacy](https://aistudio.google.com/docs/pricing)).

## Decision

1. Support **three direct cloud providers only**: OpenAI (`Sign in with ChatGPT` OAuth PKCE + API Key), Google Gemini (API Key with free-tier privacy warning), and Anthropic Claude (API Key). Exclude OpenRouter and local LLMs (`Ollama`, `LM Studio`).
2. Never hardcode static model lists in the UI. Query live `/models` endpoints (`GET /v1/models` handling both `models[]` and `data[]`, `GET /v1beta/models`, and `GET /v1/models` with `anthropic-version: 2026-10-21`) and filter in `qlevel-ai::model_filter` to keep Multimodal vision/PDF models only, stripping `tts`, `stt`, `whisper`, `audio`, `realtime`, `live`, `embedding`, `imagen`, `dall-e`, `moderation`, retired, and dated snapshot aliases.

## Consequences

- Direct TLS connection from the engineer's workstation to the chosen provider with native corporate root-CA support (`rustls-tls-native-roots`).
- Future flagship models appear automatically in `Advanced Options` without requiring a `qlevel` binary update.
