//! Normalized intermediate document models produced by `qlevel-parser`.

use serde::{Deserialize, Serialize};

/// Heuristic role assigned on initial file drop or clipboard triage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentClassification {
    MasterBoq,
    VendorQuote,
    SpecOrAvl,
}

/// Single raw row extracted from an Excel worksheet or clipboard table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawRow {
    /// 1-based worksheet row number matching Microsoft Excel row labels.
    pub row_number: u32,
    /// `true` when `xl/worksheets/sheet*.xml` marked `<row hidden="1">`.
    pub is_hidden: bool,
    /// Trimmed string representations of cells across columns.
    pub cells: Vec<String>,
}

/// Extracted worksheet from an Excel workbook or pasted table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedSheet {
    pub sheet_name: String,
    pub hidden_sheet: bool,
    pub rows: Vec<RawRow>,
    pub hidden_row_numbers: Vec<u32>,
}

/// Extracted page summary from a PDF tender document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedPdfPage {
    /// 1-based page number.
    pub page_number: u32,
    pub extracted_text: String,
    /// `true` when page contains fewer than 40 non-whitespace characters (scanned image page).
    pub needs_rasterization: bool,
}

/// Unified parsed representation with canonical `BLAKE3` content digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedDocument {
    pub file_name: String,
    pub blake3_hex: String,
    pub suggested_role: DocumentClassification,
    pub detected_vendor_name: Option<String>,
    pub sheets: Vec<ParsedSheet>,
    pub pdf_pages: Vec<ParsedPdfPage>,
}

impl ParsedDocument {
    /// Returns `true` when one or more PDF pages require WebView `<canvas>` rasterization.
    #[must_use]
    pub fn requires_pdf_rasterization(&self) -> bool {
        self.pdf_pages.iter().any(|p| p.needs_rasterization)
    }

    /// Returns 1-based page numbers requiring rasterization.
    #[must_use]
    pub fn pages_needing_rasterization(&self) -> Vec<u32> {
        self.pdf_pages
            .iter()
            .filter(|p| p.needs_rasterization)
            .map(|p| p.page_number)
            .collect()
    }

    /// Counts non-empty rows across all parsed sheets.
    #[must_use]
    pub fn total_tabular_rows(&self) -> usize {
        self.sheets.iter().map(|s| s.rows.len()).sum()
    }
}

/// Detected 0-based column indices inside a BOQ or vendor quotation table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedColumnMap {
    pub header_row_index: usize,
    pub code_col: Option<usize>,
    pub description_col: usize,
    pub unit_col: Option<usize>,
    pub qty_col: Option<usize>,
    pub rate_col: Option<usize>,
    pub total_col: Option<usize>,
}
