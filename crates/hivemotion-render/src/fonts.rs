//! Text measurement from the brand's own font files, so layout (fitting, baselines) is decided
//! before a frame is drawn and is the same on every machine.

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use fframes::ttf_parser::Face;

#[derive(Debug, Clone)]
pub struct FontMetrics {
    pub family: String,
    pub weight: u16,
    units_per_em: f64,
    /// Ascender and descender, as a fraction of the font size (descender positive).
    pub ascent: f64,
    pub descent: f64,
    advances: Vec<(char, f64)>,
    fallback_advance: f64,
}

impl FontMetrics {
    pub fn load(file: &Path, family: &str) -> Result<FontMetrics> {
        let data = std::fs::read(file).with_context(|| format!("reading {}", file.display()))?;
        let face =
            Face::parse(&data, 0).map_err(|e| anyhow!("{} is not a font: {e}", file.display()))?;
        let upem = face.units_per_em() as f64;
        let mut advances = Vec::new();
        for c in (' '..='~').chain("–—’“”…·•→".chars()) {
            if let Some(g) = face.glyph_index(c) {
                advances.push((c, face.glyph_hor_advance(g).unwrap_or(0) as f64 / upem));
            }
        }
        let fallback = advances
            .iter()
            .find(|(c, _)| *c == 'n')
            .map(|(_, a)| *a)
            .unwrap_or(0.55);
        let (asc, desc) = if face
            .tables()
            .os2
            .is_some_and(|t| t.use_typographic_metrics())
        {
            (face.typographic_ascender(), face.typographic_descender())
        } else {
            (Some(face.ascender()), Some(face.descender()))
        };
        Ok(FontMetrics {
            family: family.to_string(),
            weight: face.weight().to_number(),
            units_per_em: upem,
            ascent: asc.unwrap_or(face.ascender()) as f64 / upem,
            descent: -(desc.unwrap_or(face.descender()) as f64) / upem,
            advances,
            fallback_advance: fallback,
        })
    }

    /// Width of `text` at `size` px with `tracking` em of letter spacing.
    pub fn width(&self, text: &str, size: f64, tracking: f64) -> f64 {
        let mut w = 0.0;
        let mut n = 0;
        for c in text.chars() {
            let adv = self
                .advances
                .iter()
                .find(|(k, _)| *k == c)
                .map(|(_, a)| *a)
                .unwrap_or(self.fallback_advance);
            w += adv * size;
            n += 1;
        }
        if n > 1 {
            w += tracking * size * (n - 1) as f64;
        }
        let _ = self.units_per_em;
        w
    }

    /// Baseline of a line whose CSS line box starts at `top`, the way a browser places it.
    pub fn baseline(&self, top: f64, size: f64, line_height: f64) -> f64 {
        let content = (self.ascent + self.descent) * size;
        top + (line_height * size - content) / 2.0 + self.ascent * size
    }
}

/// The font file among `files` whose weight is closest to `weight`.
pub fn pick(files: &[std::path::PathBuf], family: &str, weight: u16) -> Result<FontMetrics> {
    let mut best: Option<FontMetrics> = None;
    for f in files {
        let m = FontMetrics::load(f, family)?;
        let better = match &best {
            None => true,
            Some(b) => {
                (m.weight as i32 - weight as i32).abs() < (b.weight as i32 - weight as i32).abs()
            }
        };
        if better {
            best = Some(m);
        }
    }
    best.ok_or_else(|| anyhow!("no font files for {family}"))
}
