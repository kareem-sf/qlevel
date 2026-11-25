//! Pure-Rust PDF page counter, text extractor, and scanned-page detector (`lopdf`).

use lopdf::Document;

use crate::error::{ParserError, ParserResult};
use crate::header_detector::classify_document_role;
use crate::model::{DocumentClassification, ParsedDocument, ParsedPdfPage};

/// Minimum non-whitespace character count to treat a PDF page as digital text rather than a scan.
const DIGITAL_TEXT_CHAR_THRESHOLD: usize = 40;

/// Inspects a PDF buffer in pure Rust, extracting digital text per page and marking
/// scanned/image-only pages (`needs_rasterization = true`) for WebView `pdfjs-dist` rendering.
///
/// # Errors
/// Returns [`ParserError::PasswordProtected`] if encrypted or [`ParserError::PdfCorrupted`] if malformed.
pub fn inspect_pdf_bytes(file_name: &str, bytes: &[u8]) -> ParserResult<ParsedDocument> {
    let blake3_hex = blake3::hash(bytes).to_hex().to_string();

    let doc = Document::load_mem(bytes).map_err(|err| ParserError::PdfCorrupted {
        file_name: file_name.to_owned(),
        reason: err.to_string(),
    })?;

    if doc.is_encrypted() {
        return Err(ParserError::PasswordProtected {
            file_name: file_name.to_owned(),
        });
    }

    let pages_map = doc.get_pages();
    if pages_map.is_empty() {
        return Err(ParserError::PdfCorrupted {
            file_name: file_name.to_owned(),
            reason: "PDF contains zero pages".to_owned(),
        });
    }

    let mut pdf_pages = Vec::with_capacity(pages_map.len());
    let mut sample_text = String::new();

    for page_num in pages_map.keys().copied() {
        let extracted_text = doc
            .extract_text(&[page_num])
            .unwrap_or_default()
            .trim()
            .to_owned();
        let non_ws_chars = extracted_text.chars().filter(|c| !c.is_whitespace()).count();
        let needs_rasterization = non_ws_chars < DIGITAL_TEXT_CHAR_THRESHOLD;

        if sample_text.len() < 1500 && !extracted_text.is_empty() {
            sample_text.push_str(&extracted_text);
            sample_text.push('\n');
        }

        pdf_pages.push(ParsedPdfPage {
            page_number: page_num,
            extracted_text,
            needs_rasterization,
        });
    }

    let suggested_role = classify_document_role(file_name, &sample_text);
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(base, _)| base)
        .replace(['_', '-'], " ")
        .trim()
        .to_owned();
    let detected_vendor_name =
        if suggested_role == DocumentClassification::VendorQuote && !stem.is_empty() {
            Some(stem)
        } else {
            None
        };

    Ok(ParsedDocument {
        file_name: file_name.to_owned(),
        blake3_hex,
        suggested_role,
        detected_vendor_name,
        sheets: Vec::new(),
        pdf_pages,
    })
}
