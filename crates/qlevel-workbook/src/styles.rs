//! Executive quantity-surveying formatting palette for `rust_xlsxwriter`.

use qlevel_domain::CurrencyCode;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder};

/// Reusable cell formats for commercial and technical comparison worksheets.
pub struct WorkbookStylePalette {
    pub title_banner: Format,
    pub subtitle_meta: Format,
    pub table_header: Format,
    pub vendor_group_header: Format,
    pub text_cell: Format,
    pub text_center: Format,
    pub qty_cell: Format,
    pub money_cell: Format,
    pub money_bold: Format,
    pub pct_cell: Format,
    pub lowest_winner_money: Format,
    pub lowest_winner_text: Format,
    pub yellow_merged_money: Format,
    pub yellow_merged_text: Format,
    pub risk_alert_money: Format,
    pub risk_alert_text: Format,
}

impl WorkbookStylePalette {
    #[must_use]
    pub fn new(currency: CurrencyCode) -> Self {
        let money_fmt = currency.excel_number_format();

        let title_banner = Format::new()
            .set_bold()
            .set_font_size(14.0)
            .set_font_color(Color::RGB(0xFFFFFF))
            .set_background_color(Color::RGB(0x0F172A))
            .set_align(FormatAlign::VerticalCenter);

        let subtitle_meta = Format::new()
            .set_font_size(10.0)
            .set_font_color(Color::RGB(0x334155))
            .set_background_color(Color::RGB(0xF1F5F9))
            .set_border(FormatBorder::Thin);

        let table_header = Format::new()
            .set_bold()
            .set_font_size(10.5)
            .set_font_color(Color::RGB(0xFFFFFF))
            .set_background_color(Color::RGB(0x1E293B))
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_text_wrap()
            .set_border(FormatBorder::Thin);

        let vendor_group_header = Format::new()
            .set_bold()
            .set_font_size(11.0)
            .set_font_color(Color::RGB(0xFFFFFF))
            .set_background_color(Color::RGB(0x0F766E))
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);

        let text_cell = Format::new()
            .set_font_size(10.0)
            .set_align(FormatAlign::VerticalCenter)
            .set_text_wrap()
            .set_border(FormatBorder::Thin);

        let text_center = Format::new()
            .set_font_size(10.0)
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);

        let qty_cell = Format::new()
            .set_font_size(10.0)
            .set_num_format("#,##0.00")
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);

        let money_cell = Format::new()
            .set_font_size(10.0)
            .set_num_format(money_fmt)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);

        let money_bold = Format::new()
            .set_bold()
            .set_font_size(10.5)
            .set_num_format(money_fmt)
            .set_background_color(Color::RGB(0xF8FAFC))
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);

        let pct_cell = Format::new()
            .set_font_size(10.0)
            .set_num_format("0.00\"%\"")
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);

        let lowest_winner_money = Format::new()
            .set_bold()
            .set_font_size(10.5)
            .set_num_format(money_fmt)
            .set_font_color(Color::RGB(0x166534))
            .set_background_color(Color::RGB(0xDCFCE7))
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);

        let lowest_winner_text = Format::new()
            .set_bold()
            .set_font_size(10.0)
            .set_font_color(Color::RGB(0x166534))
            .set_background_color(Color::RGB(0xDCFCE7))
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);

        let yellow_merged_money = Format::new()
            .set_bold()
            .set_font_size(10.0)
            .set_num_format(money_fmt)
            .set_font_color(Color::RGB(0x92400E))
            .set_background_color(Color::RGB(0xFEF3C7))
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);

        let yellow_merged_text = Format::new()
            .set_font_size(9.5)
            .set_font_color(Color::RGB(0x92400E))
            .set_background_color(Color::RGB(0xFEF3C7))
            .set_align(FormatAlign::VerticalCenter)
            .set_text_wrap()
            .set_border(FormatBorder::Thin);

        let risk_alert_money = Format::new()
            .set_bold()
            .set_font_size(10.0)
            .set_num_format(money_fmt)
            .set_font_color(Color::RGB(0x991B1B))
            .set_background_color(Color::RGB(0xFEE2E2))
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin);

        let risk_alert_text = Format::new()
            .set_font_size(9.5)
            .set_font_color(Color::RGB(0x991B1B))
            .set_background_color(Color::RGB(0xFEE2E2))
            .set_align(FormatAlign::VerticalCenter)
            .set_text_wrap()
            .set_border(FormatBorder::Thin);

        Self {
            title_banner,
            subtitle_meta,
            table_header,
            vendor_group_header,
            text_cell,
            text_center,
            qty_cell,
            money_cell,
            money_bold,
            pct_cell,
            lowest_winner_money,
            lowest_winner_text,
            yellow_merged_money,
            yellow_merged_text,
            risk_alert_money,
            risk_alert_text,
        }
    }
}
