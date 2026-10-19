use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The look shared by every film of one product.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    pub name: String,
    pub colors: Colors,
    pub fonts: Fonts,
    /// Path of the mark as SVG, relative to brand.yaml.
    pub mark: Option<String>,
    pub caption: CaptionStyle,
}

/// Colours as `#rrggbb`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Colors {
    pub background: String,
    pub text: String,
    pub text_dim: String,
    pub text_faint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Fonts {
    /// Captions and titles.
    pub sans: Font,
    /// Commands and labels.
    pub mono: Font,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Font {
    pub family: String,
    /// Font files (TTF or OTF), relative to brand.yaml.
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionStyle {
    pub weight: u16,
    /// Letter spacing in em.
    pub tracking: f64,
    /// Line height as a multiple of the font size.
    pub line_height: f64,
}

pub fn is_hex_color(text: &str) -> bool {
    text.len() == 7 && text.starts_with('#') && text[1..].chars().all(|c| c.is_ascii_hexdigit())
}
