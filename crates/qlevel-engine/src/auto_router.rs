//! Automatic (`Auto` mode) Multimodal model & reasoning effort router.
//!
//! Routes clean digital tabular batches to fast-tier Multimodal models at `Low` effort,
//! and escalates scanned images, exclusion pages, or dense merged rows to flagship
//! reasoning models at `High` effort.

use qlevel_domain::{AiConfig, EffortLevel, ModelInfo};

use crate::error::{EngineError, EngineResult};

/// Complexity signals observed on an individual document batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchComplexitySignals {
    pub has_scanned_images: bool,
    pub has_exclusion_or_terms_text: bool,
    pub has_hidden_excel_rows: bool,
    pub lacks_master_boq: bool,
}

impl BatchComplexitySignals {
    /// Returns `true` when the batch warrants flagship reasoning escalation in `Auto` mode.
    #[must_use]
    pub const fn requires_deep_reasoning(self) -> bool {
        self.has_scanned_images
            || self.has_exclusion_or_terms_text
            || self.has_hidden_excel_rows
            || self.lacks_master_boq
    }
}

/// Concrete model ID and effort selected for a batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedModelPlan {
    pub model_id: String,
    pub effort: EffortLevel,
}

/// Resolves the concrete Multimodal model and `EffortLevel` for a batch from live `available_models`.
///
/// # Errors
/// Returns [`EngineError::NoAvailableModels`] if `available_models` is empty.
pub fn route_batch_model_and_effort(
    ai_config: &AiConfig,
    available_models: &[ModelInfo],
    signals: BatchComplexitySignals,
) -> EngineResult<RoutedModelPlan> {
    if available_models.is_empty() {
        return Err(EngineError::NoAvailableModels);
    }

    let is_manual_model = ai_config
        .model_id
        .as_deref()
        .is_some_and(|id| !id.trim().is_empty() && !id.eq_ignore_ascii_case("auto"));

    let chosen_model =
        if let Some(manual_id) = ai_config.model_id.as_deref().filter(|_| is_manual_model) {
            available_models
                .iter()
                .find(|m| m.id == manual_id)
                .or_else(|| available_models.first())
                .ok_or(EngineError::NoAvailableModels)?
        } else if signals.requires_deep_reasoning() {
            // Prefer flagship non-fast-tier reasoning model
            available_models
                .iter()
                .find(|m| !m.is_fast_tier)
                .or_else(|| available_models.first())
                .ok_or(EngineError::NoAvailableModels)?
        } else {
            // Clean digital spreadsheet batch -> fast Multimodal tier if available
            available_models
                .iter()
                .find(|m| m.is_fast_tier)
                .or_else(|| available_models.first())
                .ok_or(EngineError::NoAvailableModels)?
        };

    let chosen_effort = match ai_config.effort {
        EffortLevel::Auto => {
            if signals.requires_deep_reasoning() {
                if chosen_model.supported_efforts.contains(&EffortLevel::High) {
                    EffortLevel::High
                } else {
                    EffortLevel::Medium
                }
            } else if chosen_model.supported_efforts.contains(&EffortLevel::Low) {
                EffortLevel::Low
            } else {
                EffortLevel::Medium
            }
        }
        explicit => {
            if chosen_model.supported_efforts.contains(&explicit) {
                explicit
            } else {
                EffortLevel::Medium
            }
        }
    };

    Ok(RoutedModelPlan {
        model_id: chosen_model.id.clone(),
        effort: chosen_effort,
    })
}
