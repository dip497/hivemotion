//! The brand mark: the filled paths of an SVG file, redrawn inside the frame.

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use fframes::roxmltree;

#[derive(Debug, Clone)]
pub struct MarkPath {
    pub d: String,
    pub fill: String,
    pub opacity: f64,
}

#[derive(Debug, Clone)]
pub struct Mark {
    pub view_x: f64,
    pub view_y: f64,
    pub view_width: f64,
    pub view_height: f64,
    pub paths: Vec<MarkPath>,
}

impl Mark {
    pub fn load(file: &Path) -> Result<Mark> {
        let text =
            std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
        let doc = roxmltree::Document::parse(&text)
            .map_err(|e| anyhow!("{} is not SVG: {e}", file.display()))?;
        let root = doc.root_element();
        let view: Vec<f64> = root
            .attribute("viewBox")
            .unwrap_or("0 0 24 24")
            .split([' ', ','])
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();
        if view.len() != 4 {
            return Err(anyhow!(
                "{}: the viewBox needs four numbers",
                file.display()
            ));
        }
        let mut paths = Vec::new();
        for node in root.descendants().filter(|n| n.has_tag_name("path")) {
            let Some(d) = node.attribute("d") else {
                continue;
            };
            let mut fill = None;
            let mut opacity = 1.0;
            for a in node.ancestors().filter(|n| n.is_element()) {
                if fill.is_none() {
                    fill = a.attribute("fill").map(str::to_string);
                }
                if let Some(o) = a.attribute("opacity").and_then(|o| o.parse::<f64>().ok()) {
                    opacity *= o;
                }
            }
            paths.push(MarkPath {
                d: d.to_string(),
                fill: fill.unwrap_or_else(|| "#ffffff".into()),
                opacity,
            });
        }
        if paths.is_empty() {
            return Err(anyhow!("{} has no <path> to draw", file.display()));
        }
        Ok(Mark {
            view_x: view[0],
            view_y: view[1],
            view_width: view[2],
            view_height: view[3],
            paths,
        })
    }
}
