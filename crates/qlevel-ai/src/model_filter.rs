//! Live `/models` filtering retaining ONLY current Multimodal (text + vision/PDF) models.

use qlevel_domain::{EffortLevel, ModelInfo, ProviderKind};

/// Normalized intermediate record parsed from a provider's live `/models` payload.
#[derive(Debug, Clone)]
pub struct RawProviderModel {
    pub id: String,
    pub display_name: Option<String>,
    pub supports_vision_hint: Option<bool>,
    pub supports_generation_hint: Option<bool>,
    pub is_deprecated: bool,
}

const EXCLUDED_SUBSTRINGS: &[&str] = &[
    "tts",
    "stt",
    "whisper",
    "transcribe",
    "audio",
    "realtime",
    "live",
    "embed",
    "embedding",
    "imagen",
    "dall-e",
    "gpt-image",
    "sora",
    "veo",
    "moderation",
    "omni-moderation",
    "babbage",
    "davinci",
    "gpt-3.5",
    "gpt-4-0",
    "gpt-4-turbo",
    "codex-mini",
    "computer-use",
    "search-preview",
    "aqa",
    "text-bison",
    "chat-bison",
    "gemini-1.0",
    "gemini-1.5",
    "gemini-2.0",
    "claude-2",
    "claude-instant",
    "claude-3-haiku-2024",
    "claude-3-sonnet-2024",
    "claude-3-opus-2024",
];

/// Detects dated snapshot suffixes like `-2024-08-06`, `-20250514`, or `-001` when canonical
/// evergreen aliases (`claude-sonnet-5-5`, `gpt-6.2`, `gemini-3.8-flash`) are preferred,
/// while keeping Claude/Gemini canonical IDs if no undated alias exists in the same response.
fn has_dated_snapshot_suffix(id: &str) -> bool {
    let lower = id.to_lowercase();
    // Match YYYY-MM-DD at end
    if lower.len() > 11 {
        let tail = &lower[lower.len() - 10..];
        let bytes = tail.as_bytes();
        if bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[0..4].iter().all(u8::is_ascii_digit)
            && bytes[5..7].iter().all(u8::is_ascii_digit)
            && bytes[8..10].iter().all(u8::is_ascii_digit)
        {
            return true;
        }
    }
    // Match -YYYYMMDD at end
    if lower.len() > 9 {
        let tail = &lower[lower.len() - 9..];
        let bytes = tail.as_bytes();
        if bytes[0] == b'-'
            && (tail.starts_with("-202") || tail.starts_with("-203"))
            && bytes[1..].iter().all(u8::is_ascii_digit)
        {
            return true;
        }
    }
    false
}

fn strip_dated_suffix(id: &str) -> &str {
    if has_dated_snapshot_suffix(id) {
        if id.len() > 11 && id.as_bytes()[id.len() - 11] == b'-' {
            return &id[..id.len() - 11];
        }
        if id.len() > 9 && id.as_bytes()[id.len() - 9] == b'-' {
            return &id[..id.len() - 9];
        }
    }
    id
}

fn is_multimodal_family(provider: ProviderKind, lower_id: &str) -> bool {
    match provider {
        ProviderKind::OpenAi => {
            lower_id.starts_with("gpt-4.1")
                || lower_id.starts_with("gpt-4o")
                || lower_id.starts_with("gpt-5")
                || lower_id.starts_with("gpt-6")
                || lower_id.starts_with("o3")
                || lower_id.starts_with("o4")
        }
        ProviderKind::Gemini => {
            lower_id.starts_with("gemini-2.5")
                || lower_id.starts_with("gemini-3")
                || lower_id.starts_with("gemini-4")
        }
        ProviderKind::Claude => {
            lower_id.starts_with("claude-3-5")
                || lower_id.starts_with("claude-3-7")
                || lower_id.starts_with("claude-sonnet-4")
                || lower_id.starts_with("claude-opus-4")
                || lower_id.starts_with("claude-haiku-4")
                || lower_id.starts_with("claude-sonnet-5")
                || lower_id.starts_with("claude-opus-5")
                || lower_id.starts_with("claude-haiku-5")
        }
    }
}

fn is_fast_tier_model(lower_id: &str) -> bool {
    lower_id.contains("mini")
        || lower_id.contains("nano")
        || lower_id.contains("flash")
        || lower_id.contains("haiku")
}

fn supported_efforts_for(provider: ProviderKind, lower_id: &str) -> Vec<EffortLevel> {
    match provider {
        ProviderKind::OpenAi => {
            if lower_id.starts_with("gpt-4o") || lower_id.starts_with("gpt-4.1") {
                vec![EffortLevel::Auto, EffortLevel::Medium]
            } else {
                vec![
                    EffortLevel::Auto,
                    EffortLevel::Low,
                    EffortLevel::Medium,
                    EffortLevel::High,
                ]
            }
        }
        ProviderKind::Gemini => vec![
            EffortLevel::Auto,
            EffortLevel::Low,
            EffortLevel::Medium,
            EffortLevel::High,
        ],
        ProviderKind::Claude => vec![
            EffortLevel::Auto,
            EffortLevel::Low,
            EffortLevel::Medium,
            EffortLevel::High,
        ],
    }
}

fn humanize_model_name(id: &str, display_name: Option<&str>) -> String {
    if let Some(name) = display_name {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_owned();
        }
    }
    id.to_owned()
}

/// Filters a raw live `/models` list down to clean, non-deprecated Multimodal models only.
#[must_use]
pub fn filter_live_multimodal_models(
    provider: ProviderKind,
    raw_models: Vec<RawProviderModel>,
) -> Vec<ModelInfo> {
    let all_ids: std::collections::HashSet<String> =
        raw_models.iter().map(|m| m.id.to_lowercase()).collect();

    let mut filtered = Vec::new();
    for raw in raw_models {
        if raw.is_deprecated {
            continue;
        }
        if raw.supports_generation_hint == Some(false) {
            continue;
        }
        if raw.supports_vision_hint == Some(false) {
            continue;
        }

        let clean_id = raw.id.trim().trim_start_matches("models/").to_owned();
        let lower_id = clean_id.to_lowercase();

        if EXCLUDED_SUBSTRINGS.iter().any(|bad| lower_id.contains(bad)) {
            continue;
        }
        if !is_multimodal_family(provider, &lower_id) {
            continue;
        }

        // Deduplicate dated snapshots when the undated canonical alias is also present
        if has_dated_snapshot_suffix(&lower_id) {
            let base = strip_dated_suffix(&lower_id);
            if base != lower_id && all_ids.contains(base) {
                continue;
            }
        }

        let is_fast_tier = is_fast_tier_model(&lower_id);
        filtered.push(ModelInfo {
            display_name: humanize_model_name(&clean_id, raw.display_name.as_deref()),
            supported_efforts: supported_efforts_for(provider, &lower_id),
            id: clean_id,
            provider,
            supports_vision: true,
            supports_pdf_input: true,
            supports_structured_output: true,
            is_fast_tier,
        });
    }

    // Sort flagship reasoning models first, then fast-tier models, alphabetically deterministic
    filtered.sort_by(|a, b| {
        a.is_fast_tier
            .cmp(&b.is_fast_tier)
            .then_with(|| b.id.cmp(&a.id))
    });
    filtered.dedup_by(|a, b| a.id == b.id);
    filtered
}
