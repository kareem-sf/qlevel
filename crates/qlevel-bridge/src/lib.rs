//! Desktop integration bridge for `qlevel`.
//!
//! Isolates OS credential encryption (`vault`), portable user-space self-updates (`updater`),
//! strongly typed IPC contracts (`dto`), and session coordination (`state`, `commands`)
//! cleanly from both `qlevel-engine` and the thin Tauri binary shell.

#![forbid(unsafe_code)]

pub mod commands;
pub mod dto;
pub mod error;
pub mod state;
pub mod updater;
pub mod vault;

pub use commands::BridgeEventEmitter;
pub use dto::{
    AmbiguityDecisionInput, AmbiguityPromptPayload, AmbiguityResolutionChoice, BridgeErrorCode,
    BridgeErrorPayload, DocumentRoleDto, ExportWorkbookInput, InputSourceKindDto,
    LevelingProgressEvent, LevelingSessionResultDto, PdfRasterRequestEvent, ProgressStageDto,
    ProviderAuthStatusDto, RenderedPageImageDto, SaveApiKeyInput, StagedInputDocumentDto,
    StartLevelingSessionInput, SubmitRenderedPdfPagesInput, TrapFindingSummaryDto,
    TriageClipboardInput, UpdateCheckStatusDto, VendorSummaryRowDto, WorkbookExportOutcomeDto,
};
pub use error::{BridgeError, BridgeResult};
pub use state::BridgeState;
pub use updater::{check_or_apply_portable_update, verify_sha256_hex};
pub use vault::{
    EnvelopeVault, InMemoryMasterKeyBackend, MasterKeyBackend, OsKeyringMasterBackend,
};
