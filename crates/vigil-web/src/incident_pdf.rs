//! Minimal printable PDF (built-in Helvetica; ASCII-safe text).

use printpdf::{BuiltinFont, Mm, PdfDocument};
use std::io::{BufWriter, Cursor};

fn sanitize_line(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii() && c != '\r' {
                c
            } else {
                '?'
            }
        })
        .collect::<String>()
        .replace('\n', " | ")
}

fn truncate(s: &str, max: usize) -> String {
    let t = sanitize_line(s);
    if t.len() <= max {
        t
    } else {
        format!("{}...", &t[..max])
    }
}

/// Build a one-page incident summary PDF.
pub fn build_incident_pdf(
    id: &str,
    title: &str,
    status: &str,
    severity: &str,
    tenant: &str,
    verification: &str,
    extra: &str,
) -> Result<Vec<u8>, String> {
    let (doc, page1, layer1) = PdfDocument::new(
        &format!("incident-{id}"),
        Mm(210.0),
        Mm(297.0),
        "L1",
    );
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| e.to_string())?;
    let layer = doc.get_page(page1).get_layer(layer1);

    let mut y = 285.0_f64;
    let lines: Vec<(String, f64)> = vec![
        ("Vigil incident export (PDF)".to_string(), 12.0),
        (format!("ID: {id}"), 10.0),
        (format!("Title: {title}"), 10.0),
        (
            format!("Status: {status}  Severity: {severity}  Tenant: {tenant}"),
            10.0,
        ),
        (format!("Verification: {verification}"), 10.0),
        ("---".to_string(), 10.0),
    ];

    for (text, size) in lines {
        let chunk = truncate(&text, 120);
        layer.use_text(chunk, size, Mm(15.0), Mm(y), &font);
        y -= size * 0.55 + 5.0;
    }

    let body = truncate(extra, 8000);
    for part in body.as_bytes().chunks(900) {
        let s = std::str::from_utf8(part).unwrap_or("");
        let chunk = truncate(s, 900);
        if y < 35.0 {
            break;
        }
        layer.use_text(chunk, 9.0, Mm(15.0), Mm(y), &font);
        y -= 12.0;
    }

    let mut w = BufWriter::new(Cursor::new(Vec::new()));
    doc.save(&mut w).map_err(|e| e.to_string())?;
    w.into_inner()
        .map_err(|_| "buffer".to_string())
        .map(|c| c.into_inner())
}
