//! Deterministic Commercial & Technical Bid Leveling (`CBE` + `TBE`) engine for `qlevel`.
//!
//! Enforces `#![deny(clippy::float_arithmetic)]` across every module: all commercial rates,
//! quantities, VAT conversions, Provisional Sum locks, merged-row splits, and missing-scope
//! plugs execute strictly in fixed-point `DecimalValue`.

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

pub mod auto_router;
pub mod checkpoint_db;
pub mod commercial_calc;
pub mod error;
pub mod merged_resolver;
pub mod pipeline;
pub mod synthetic_boq;
pub mod technical_eval;

pub use auto_router::{route_batch_model_and_effort, BatchComplexitySignals, RoutedModelPlan};
pub use checkpoint_db::{compute_batch_blake3_key, CheckpointStore};
pub use commercial_calc::evaluate_commercial_leveling;
pub use error::{EngineError, EngineResult};
pub use merged_resolver::{apply_merged_group_resolutions, build_initial_alignment_groups};
pub use pipeline::{
    evaluate_quotes_deterministic, run_extraction_and_leveling_pipeline, EngineProgressUpdate,
    PipelinePhase,
};
pub use synthetic_boq::{build_synthetic_master_boq, RawVendorCandidateItem, SyntheticBoqOutcome};
pub use technical_eval::evaluate_technical_traps;
