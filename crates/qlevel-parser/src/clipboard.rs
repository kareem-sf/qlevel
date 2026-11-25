//! Clipboard table parser supporting Excel/email HTML `<table>` and TSV/CSV paste (`Ctrl+V`).

use quick_xml::escape::unescape;
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;

use crate::error::{ParserError, ParserResult};
use crate::header_detector::classify_document_role;
use crate::model::{ParsedDocument, ParsedSheet, RawRow};

fn parse_html_table_fragment(html: &str) -> Vec<RawRow> {
    let mut reader = XmlReader::from_str(html);
    reader.config_mut().check_end_names = false;
    reader.config_mut().trim_text(true);

    let mut rows = Vec::new();
    let mut in_row = false;
    let mut in_cell = false;
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let lower = e.local_name().as_ref().to_lowercase();
                if lower == "tr" {
                    in_row = true;
                    current_row.clear();
                } else if in_row && (lower == "td" || lower == "th") {
                    in_cell = true;
                    current_cell.clear();
                }
            }
            Ok(Event::Text(t)) if in_cell => {
                if let Ok(decoded) = unescape(t.as_ref()) {
                    if !current_cell.is_empty() {
                        current_cell.push(' ');
                    }
                    current_cell.push_str(decoded.trim());
                }
            }
            Ok(Event::End(e)) => {
                let lower = e.local_name().as_ref().to_lowercase();
                if (lower == "td" || lower == "th") && in_cell {
                    current_row.push(current_cell.trim().to_owned());
                    current_cell.clear();
                    in_cell = false;
                } else if lower == "tr" && in_row {
                    if current_row.iter().any(|c| !c.is_empty()) {
                        let row_number = (rows.len() as u32) + 1;
                        rows.push(RawRow {
                            row_number,
                            is_hidden: false,
                            cells: current_row.clone(),
                        });
                    }
                    current_row.clear();
                    in_row = false;
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    rows
}

fn parse_delimited_text(plain_text: &str) -> Vec<RawRow> {
    let mut rows = Vec::new();
    let use_tabs = plain_text.lines().take(5).any(|line| line.contains('\t'));

    for line in plain_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let cells: Vec<String> = if use_tabs {
            line.split('\t').map(|s| s.trim().to_owned()).collect()
        } else if line.contains('|') {
            line.split('|')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.chars().all(|c| c == '-'))
                .collect()
        } else {
            line.split(',').map(|s| s.trim().to_owned()).collect()
        };

        if cells.iter().any(|c| !c.is_empty()) {
            let row_number = (rows.len() as u32) + 1;
            rows.push(RawRow {
                row_number,
                is_hidden: false,
                cells,
            });
        }
    }
    rows
}

/// Parses clipboard HTML or TSV/CSV text pasted via `Ctrl+V` into a `ParsedDocument`.
///
/// # Errors
/// Returns [`ParserError::EmptyTable`] if neither HTML nor plain text yields tabular rows.
pub fn parse_clipboard_table(
    label: &str,
    html: Option<&str>,
    plain_text: Option<&str>,
) -> ParserResult<ParsedDocument> {
    let mut rows = html
        .filter(|h| h.to_lowercase().contains("<tr"))
        .map_or_else(Vec::new, parse_html_table_fragment);

    if rows.is_empty() {
        if let Some(txt) = plain_text {
            rows = parse_delimited_text(txt);
        }
    }

    if rows.is_empty() {
        return Err(ParserError::EmptyTable {
            source_name: label.to_owned(),
        });
    }

    let mut hash_input = blake3::Hasher::new();
    for r in &rows {
        for c in &r.cells {
            hash_input.update(c.as_bytes());
            hash_input.update(b"\t");
        }
        hash_input.update(b"\n");
    }
    let blake3_hex = hash_input.finalize().to_hex().to_string();

    let sample_text: String = rows
        .iter()
        .take(8)
        .flat_map(|r| r.cells.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ");
    let suggested_role = classify_document_role(label, &sample_text);

    Ok(ParsedDocument {
        file_name: label.to_owned(),
        blake3_hex,
        suggested_role,
        detected_vendor_name: Some(label.to_owned()),
        sheets: vec![ParsedSheet {
            sheet_name: "Pasted_Table".to_owned(),
            hidden_sheet: false,
            rows,
            hidden_row_numbers: Vec::new(),
        }],
        pdf_pages: Vec::new(),
    })
}
