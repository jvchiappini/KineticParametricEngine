use kpe_schema::fabrication::NestingSheet;

pub struct SvgExporter;

impl SvgExporter {
    pub fn new() -> Self {
        Self
    }

    pub fn export(&self, sheets: &[NestingSheet]) -> Vec<u8> {
        let mut svg = String::new();

        svg.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        svg.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 500">"#);

        for sheet in sheets {
            for piece in &sheet.pieces {
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="black" stroke-width="0.5"/>"#,
                    piece.x * 0.5,
                    piece.y * 0.5,
                    piece.width * 0.5,
                    piece.height * 0.5,
                ));
                svg.push_str(&format!(
                    r#"<text x="{}" y="{}" font-size="3">{}</text>"#,
                    (piece.x + 2.0) * 0.5,
                    (piece.y + 10.0) * 0.5,
                    piece.piece_id,
                ));
            }
        }

        svg.push_str("</svg>");
        svg.into_bytes()
    }
}

impl Default for SvgExporter {
    fn default() -> Self {
        Self::new()
    }
}
