//! Contract tests for `qlevel-ai`: Multimodal `/models` filtering, OpenAI SIWC PKCE & SSE,
//! Gemini `thought: true` filtering, and Claude `2026-10-21` payload rules.

use qlevel_ai::providers::claude::build_claude_messages_payload;
use qlevel_ai::providers::gemini::extract_gemini_non_thought_text;
use qlevel_ai::providers::openai::{
    build_openai_responses_payload, collect_openai_sse_text, parse_openai_models_response,
};
use qlevel_ai::{
    build_siwc_authorize_url, filter_live_multimodal_models, generate_pkce_pair,
    ExtractionBatchInput, RawProviderModel,
};
use qlevel_domain::{EffortLevel, ProviderKind};

#[test]
fn strips_audio_tts_embeddings_and_dated_duplicates_leaving_multimodal_only() {
    let raw = vec![
        RawProviderModel {
            id: "gpt-6.2".to_owned(),
            display_name: Some("GPT-6.2".to_owned()),
            supports_vision_hint: None,
            supports_generation_hint: Some(true),
            is_deprecated: false,
        },
        RawProviderModel {
            id: "gpt-6.2-2026-09-15".to_owned(),
            display_name: None,
            supports_vision_hint: None,
            supports_generation_hint: Some(true),
            is_deprecated: false,
        },
        RawProviderModel {
            id: "gpt-6.2-mini".to_owned(),
            display_name: None,
            supports_vision_hint: None,
            supports_generation_hint: Some(true),
            is_deprecated: false,
        },
        RawProviderModel {
            id: "gpt-4o-mini-tts".to_owned(),
            display_name: None,
            supports_vision_hint: None,
            supports_generation_hint: Some(true),
            is_deprecated: false,
        },
        RawProviderModel {
            id: "whisper-1".to_owned(),
            display_name: None,
            supports_vision_hint: None,
            supports_generation_hint: Some(true),
            is_deprecated: false,
        },
        RawProviderModel {
            id: "text-embedding-3-large".to_owned(),
            display_name: None,
            supports_vision_hint: None,
            supports_generation_hint: Some(true),
            is_deprecated: false,
        },
    ];

    let models = filter_live_multimodal_models(ProviderKind::OpenAi, raw);
    let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, vec!["gpt-6.2", "gpt-6.2-mini"]);
    assert!(!models[0].is_fast_tier);
    assert!(models[1].is_fast_tier);
}

#[test]
fn parses_both_siwc_models_array_and_platform_data_array() {
    let siwc_json = r#"{"models":[{"id":"gpt-6.2","display_name":"GPT-6.2"}]}"#;
    let api_json = r#"{"object":"list","data":[{"id":"gpt-6.2"}]}"#;

    assert_eq!(parse_openai_models_response(siwc_json).expect("siwc").len(), 1);
    assert_eq!(parse_openai_models_response(api_json).expect("api").len(), 1);
}

#[test]
fn builds_valid_siwc_pkce_url_and_streaming_responses_payload() {
    let pkce = generate_pkce_pair();
    assert_eq!(pkce.code_verifier.len(), 64);

    let url = build_siwc_authorize_url(
        "http://127.0.0.1:54123/oauth/callback",
        &pkce,
        "urn:uuid:123e4567-e89b-12d3-a456-426614174000",
        None,
    )
    .expect("auth url");
    assert!(url.contains("client_id=dynamic_agent_client"));
    assert!(url.contains("code_challenge_method=S256"));
    assert!(url.contains("ext_agent_host_id=urn%3Auuid%3A"));

    let batch = ExtractionBatchInput {
        batch_id: "B-1".to_owned(),
        vendor_name: "Alfanar MEP".to_owned(),
        master_boq_context_json: "[]".to_owned(),
        vendor_raw_text_or_rows: "1.1 Chiller 500 TR".to_owned(),
        images: Vec::new(),
        approved_vendors: vec!["Trane".to_owned()],
        custom_rules: vec!["Include seismic isolators".to_owned()],
    };

    let req = build_openai_responses_payload("gpt-6.2", EffortLevel::High, &batch);
    assert_eq!(req["store"], false);
    assert_eq!(req["stream"], true);
    assert!(req.get("temperature").is_none());
    assert_eq!(req["reasoning"]["effort"], "high");

    let sse = "data: {\"type\":\"response.output_text.delta\",\"delta\":\"{\\\"lines\\\":[]\"}\n\
               data: {\"type\":\"response.output_text.delta\",\"delta\":\",\\\"traps\\\":[]}\"}\n\
               data: [DONE]\n";
    let joined = collect_openai_sse_text(sse).expect("sse");
    assert_eq!(joined, "{\"lines\":[],\"traps\":[]}");
}

#[test]
fn claude_2026_10_21_omits_temperature_and_gemini_skips_thought_parts() {
    let batch = ExtractionBatchInput {
        batch_id: "B-2".to_owned(),
        vendor_name: "Vendor B".to_owned(),
        master_boq_context_json: "[]".to_owned(),
        vendor_raw_text_or_rows: "Row 1".to_owned(),
        images: Vec::new(),
        approved_vendors: Vec::new(),
        custom_rules: Vec::new(),
    };

    let claude_payload =
        build_claude_messages_payload("claude-opus-5-5", EffortLevel::High, &batch);
    assert!(claude_payload.get("temperature").is_none());
    assert!(claude_payload.get("top_p").is_none());
    assert_eq!(claude_payload["thinking"]["type"], "adaptive");

    let gemini_resp = r#"{
      "candidates": [{
        "content": {
          "parts": [
            {"thought": true, "text": "internal reasoning..."},
            {"text": "{\"lines\":[],\"traps\":[]}"}
          ]
        }
      }]
    }"#;
    let extracted = extract_gemini_non_thought_text(gemini_resp).expect("gemini");
    assert_eq!(extracted, "{\"lines\":[],\"traps\":[]}");
}
