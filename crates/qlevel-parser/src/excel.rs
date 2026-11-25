//! Pure-Rust Excel workbook parser combining `calamine` with `quick-xml` hidden-row detection.

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};

use calamine::{open_workbook_auto_from_rs, Data, Reader};
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;
use zip::ZipArchive;

use crate::error::{ParserError, ParserResult};
use crate::header_detector::classify_document_role;
use crate::model::{ParsedDocument, ParsedSheet, RawRow};

/// Compound File Binary (`OLE2`) magic header present on password-encrypted `.xlsx` files.
const OLE2_MAGIC: &[u8; 8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

fn is_encrypted_ole_package(file_name: &str, bytes: &[u8]) -> bool {
    let lower = file_name.to_lowercase();
    let is_modern_ooxml_ext = lower.ends_with(".xlsx") || lower.ends_with(".xlsm");
    if is_modern_ooxml_ext && bytes.starts_with(OLE2_MAGIC) {
        return true;
    }
    // Check ASCII/UTF-16LE marker for EncryptedPackage inside OLE container
    bytes
        .windows(b"EncryptedPackage".len())
        .any(|w| w == b"EncryptedPackage")
}

/// Inspects OOXML zip archive (`xl/workbook.xml` + `xl/worksheets/sheet*.xml`) to find
/// hidden sheets (`state="hidden"` / `state="veryHidden"`) and `<row r="N" hidden="1">` rows.
fn scan_ooxml_hidden_metadata(
    bytes: &[u8],
) -> (HashSet<String>, HashMap<usize, Vec<u32>>) {
    let mut hidden_sheets = HashSet::new();
    let mut hidden_rows_by_sheet_idx: HashMap<usize, Vec<u32>> = HashMap::new();

    let Ok(mut archive) = ZipArchive::new(Cursor::new(bytes)) else {
        return (hidden_sheets, hidden_rows_by_sheet_idx);
    };

    // 1. Parse xl/workbook.xml for sheet names & hidden states in declaration order
    if let Ok(mut wb_file) = archive.by_name("xl/workbook.xml") {
        let mut xml_str = String::new();
        if wb_file.read_to_string(&mut xml_str).is_ok() {
            let mut reader = XmlReader::from_str(&xml_str);
            reader.config_mut().trim_text(true);
            loop {
                match reader.read_event() {
                    Ok(Event::Empty(e) | Event::Start(e)) if e.local_name().as_ref() == b"sheet" => {
                        let mut name_opt = None;
                        let mut is_hidden = false;
                        for attr in e.attributes().flatten() {
                            match attr.key.local_name().as_ref() {
                                b"name" => {
                                    name_opt = String::from_utf8(attr.value.into_owned()).ok();
                                }
                                b"state" => {
                                    let val = attr.value;
                                    if val.as_ref() == b"hidden" || val.as_ref() == b"veryHidden" {
                                        is_hidden = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if is_hidden {
                            if let Some(sheet_name) = name_opt {
                                hidden_sheets.insert(sheet_name);
                            }
                        }
                    }
                    Ok(Event::Eof) | Err(_) => break,
                    _ => {}
                }
            }
        }
    }

    // 2. Scan xl/worksheets/sheet{1..N}.xml for <row r="12" hidden="1"> or hidden="true"
    for idx in 1..=64_usize {
        let entry_name = format!("xl/worksheets/sheet{idx}.xml");
        let Ok(mut sheet_file) = archive.by_name(&entry_name) else {
            continue;
        };
        let mut xml_str = String::new();
        if sheet_file.read_to_string(&mut xml_str).is_err() {
            continue;
        }
        let mut hidden_rows = Vec::new();
        let mut reader = XmlReader::from_str(&xml_str);
        reader.config_mut().trim_text(true);
        loop {
            match reader.read_event() {
                Ok(Event::Empty(e) | Event::Start(e)) if e.local_name().as_ref() == b"row" => {
                    let mut row_num: Option<u32> = None;
                    let mut row_hidden = false;
                    for attr in e.attributes().flatten() {
                        match attr.key.local_name().as_ref() {
                            b"r" => {
                                if let Ok(s) = std::str::from_utf8(attr.value.as_ref()) {
                                    row_num = s.parse::<u32>().ok();
                                }
                            }
                            b"hidden" => {
                                let v = attr.value;
                                if v.as_ref() == b"1" || v.as_ref() == b"true" {
                                    row_hidden = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    if row_hidden {
                        if let Some(r) = row_num {
                            hidden_rows.push(r);
                        }
                    }
                }
                Ok(Event::Eof) | Err(_) => break,
                _ => {}
            }
        }
        if !hidden_rows.is_empty() {
            hidden_rows_by_sheet_idx.insert(idx - 1, hidden_rows);
        }
    }

    (hidden_sheets, hidden_rows_by_sheet_idx)
}

fn format_calamine_cell(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_owned(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            let s = f.to_string();
            s.trim().to_owned()
        }
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.trim().to_owned(),
        Data::Error(e) => format!("#ERR:{e:?}"),
    }
}

fn infer_vendor_name_from_filename(file_name: &str) -> Option<String> {
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(base, _)| base)
        .replace(['_', '-'], " ");
    let cleaned = stem.trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_owned())
    }
}

/// Parses an Excel workbook (`.xlsx`, `.xlsm`, `.xls`, `.xlsb`) from memory bytes,
/// annotating hidden sheets and `<row hidden="1">` rows.
///
/// # Errors
/// Returns [`ParserError::PasswordProtected`] for encrypted workbooks or
/// [`ParserError::WorkbookCorrupted`] if unreadable.
pub fn parse_excel_bytes(file_name: &str, bytes: &[u8]) -> ParserResult<ParsedDocument> {
    if is_encrypted_ole_package(file_name, bytes) {
        return Err(ParserError::PasswordProtected {
            file_name: file_name.to_owned(),
        });
    }

    let blake3_hex = blake3::hash(bytes).to_hex().to_string();
    let (hidden_sheet_names, hidden_rows_by_idx) = scan_ooxml_hidden_metadata(bytes);

    let mut workbook =
        open_workbook_auto_from_rs(Cursor::new(bytes)).map_err(|err| ParserError::WorkbookCorrupted {
            file_name: file_name.to_owned(),
            reason: err.to_string(),
        })?;

    let sheet_names = workbook.sheet_names();
    let mut parsed_sheets = Vec::with_capacity(sheet_names.len());
    let mut sample_text = String::new();

    for (sheet_idx, sheet_name) in sheet_names.into_iter().enumerate() {
        let Ok(range) = workbook.worksheet_range(&sheet_name) else {
            continue;
        };
        let start_row = range.start().map_or(0, |(r, _)| r);
        let hidden_vec = hidden_rows_by_idx
            .get(&sheet_idx)
            .cloned()
            .unwrap_or_default();
        let hidden_set: HashSet<u32> = hidden_vec.iter().copied().collect();

        let mut rows = Vec::new();
        for (offset, row_slice) in range.rows().enumerate() {
            let excel_row_num = start_row + (offset as u32) + 1;
            let cells: Vec<String> = row_slice.iter().map(format_calamine_cell).collect();
            if cells.iter().all(String::is_empty) {
                continue;
            }
            if sample_text.len() < 1024 {
                for cell in &cells {
                    if !cell.is_empty() {
                        sample_text.push_str(cell);
                        sample_text.push(' ');
                    }
                }
            }
            rows.push(RawRow {
                row_number: excel_row_num,
                is_hidden: hidden_set.contains(&excel_row_num),
                cells,
            });
        }

        parsed_sheets.push(ParsedSheet {
            hidden_sheet: hidden_sheet_names.contains(&sheet_name),
            sheet_name,
            rows,
            hidden_row_numbers: hidden_vec,
        });
    }

    if parsed_sheets.iter().all(|s| s.rows.is_empty()) {
        return Err(ParserError::EmptyTable {
            source_name: file_name.to_owned(),
        });
    }

    let suggested_role = classify_document_role(file_name, &sample_text);
    let detected_vendor_name = if suggested_role == DocumentClassification::VendorQuote {
        infer_vendor_name_from_filename(file_name)
    } else {
        None
    };

    Ok(ParsedDocument {
        file_name: file_name.to_owned(),
        blake3_hex,
        suggested_role,
        detected_vendor_name,
        sheets: parsed_sheets,
        pdf_pages: Vec::new(),
    })
}
