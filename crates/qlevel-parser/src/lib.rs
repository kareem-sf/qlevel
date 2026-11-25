//! Pure-Rust document ingestion, Excel hidden-row auditing, PDF inspection,
//! clipboard table parsing, and bilingual header mapping for `qlevel`.

#![forbid(unsafe_code)]

pub mod clipboard;
pub mod error;
pub mod excel;
pub mod header_detector;
pub mod lock_probe;
pub mod model;
pub mod pdf;

pub use clipboard::parse_clipboard_table;
pub use error::{ParserError, ParserResult};
pub use excel::parse_excel_bytes;
pub use header_detector::{
    classify_document_role, detect_header_columns, extract_master_boq_from_sheet,
    normalize_arabic_digits,
};
pub use lock_probe::{probe_output_path_writable, suggest_unlocked_version_path, FileLockStatus};
pub use model::{
    DetectedColumnMap, DocumentClassification, ParsedDocument, ParsedPdfPage, ParsedSheet, RawRow,
};
pub use pdf::inspect_pdf_bytes;
