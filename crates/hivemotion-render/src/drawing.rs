//! An SVG file redrawn inside the frame: the brand mark and the layers of a graphic.
//!
//! Paths, rectangles, circles, ellipses, lines, polylines and polygons are drawn, each as a
//! path, with fill, stroke and opacity inherited from their groups and every group transform kept.
//! A group's opacity is applied to each shape in it, so shapes that overlap inside one
//! see-through group show through each other.

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use fframes::{roxmltree, Svgr};

#[derive(Debug, Clone)]
pub struct Shape {
    pub d: String,
    pub fill: String,
    pub fill_opacity: f64,
    pub stroke: String,
    pub stroke_width: f64,
    pub stroke_opacity: f64,
    pub line_cap: String,
    pub opacity: f64,
    /// Every transform from the outermost group to the shape, in SVG order.
    pub transform: String,
}

#[derive(Debug, Clone)]
pub struct Drawing {
    pub view_x: f64,
    pub view_y: f64,
    pub view_width: f64,
    pub view_height: f64,
    pub shapes: Vec<Shape>,
}

fn number(node: roxmltree::Node, name: &str) -> f64 {
    node.attribute(name)
        .map(|v| v.trim().trim_end_matches("px"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0)
}

fn points(text: &str) -> Vec<f64> {
    text.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect()
}

/// The outline of one element as path data, or None for an element that draws nothing.
fn outline(node: roxmltree::Node) -> Option<String> {
    let n = |name| number(node, name);
    match node.tag_name().name() {
        "path" => node.attribute("d").map(str::to_string),
        "rect" => {
            let (x, y, w, h) = (n("x"), n("y"), n("width"), n("height"));
            let (rx, ry) = match (node.attribute("rx"), node.attribute("ry")) {
                (None, None) => (0.0, 0.0),
                (Some(_), None) => (n("rx"), n("rx")),
                (None, Some(_)) => (n("ry"), n("ry")),
                _ => (n("rx"), n("ry")),
            };
            let (rx, ry) = (rx.min(w / 2.0), ry.min(h / 2.0));
            if rx <= 0.0 || ry <= 0.0 {
                return Some(format!("M{x} {y}H{}V{}H{x}Z", x + w, y + h));
            }
            Some(format!(
                "M{} {y}H{}A{rx} {ry} 0 0 1 {} {}V{}A{rx} {ry} 0 0 1 {} {}H{}A{rx} {ry} 0 0 1 {x} {}V{}A{rx} {ry} 0 0 1 {} {y}Z",
                x + rx,
                x + w - rx,
                x + w,
                y + ry,
                y + h - ry,
                x + w - rx,
                y + h,
                x + rx,
                y + h - ry,
                y + ry,
                x + rx,
            ))
        }
        "circle" | "ellipse" => {
            let (cx, cy) = (n("cx"), n("cy"));
            let (rx, ry) = if node.has_tag_name("circle") {
                (n("r"), n("r"))
            } else {
                (n("rx"), n("ry"))
            };
            Some(format!(
                "M{} {cy}A{rx} {ry} 0 1 0 {} {cy}A{rx} {ry} 0 1 0 {} {cy}Z",
                cx - rx,
                cx + rx,
                cx - rx
            ))
        }
        "line" => Some(format!("M{} {}L{} {}", n("x1"), n("y1"), n("x2"), n("y2"))),
        "polyline" | "polygon" => {
            let p = points(node.attribute("points")?);
            let mut d = String::new();
            for (i, [x, y]) in p.as_chunks::<2>().0.iter().enumerate() {
                d.push_str(&format!("{}{x} {y}", if i == 0 { "M" } else { "L" }));
            }
            if node.has_tag_name("polygon") {
                d.push('Z');
            }
            Some(d)
        }
        _ => None,
    }
}

impl Drawing {
    pub fn load(file: &Path) -> Result<Drawing> {
        let text =
            std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
        let doc = roxmltree::Document::parse(&text)
            .map_err(|e| anyhow!("{} is not SVG: {e}", file.display()))?;
        let root = doc.root_element();
        let view = match root.attribute("viewBox") {
            Some(v) => points(v),
            None => vec![0.0, 0.0, number(root, "width"), number(root, "height")],
        };
        if view.len() != 4 || view[2] <= 0.0 || view[3] <= 0.0 {
            return Err(anyhow!(
                "{}: give the <svg> a viewBox of four numbers, or a width and height",
                file.display()
            ));
        }
        let mut shapes = Vec::new();
        for node in root.descendants().filter(|n| n.is_element()) {
            if node.ancestors().any(|a| a.has_tag_name("defs")) {
                continue;
            }
            let Some(d) = outline(node) else {
                continue;
            };
            // nearest value wins for paint; opacity multiplies down the groups
            let mut chain: Vec<roxmltree::Node> =
                node.ancestors().filter(|a| a.is_element()).collect();
            let inherited = |name: &str| chain.iter().find_map(|a| a.attribute(name));
            let fill = inherited("fill").unwrap_or("#000000").to_string();
            let stroke = inherited("stroke").unwrap_or("none").to_string();
            let read = |name: &str, default: f64| {
                inherited(name)
                    .and_then(|v| v.trim().trim_end_matches("px").parse().ok())
                    .unwrap_or(default)
            };
            let (fill_opacity, stroke_opacity) =
                (read("fill-opacity", 1.0), read("stroke-opacity", 1.0));
            let stroke_width = read("stroke-width", 1.0);
            let line_cap = inherited("stroke-linecap").unwrap_or("butt").to_string();
            let opacity = chain
                .iter()
                .filter_map(|a| a.attribute("opacity").and_then(|o| o.parse::<f64>().ok()))
                .product();
            chain.reverse();
            let transform = chain
                .iter()
                .filter_map(|a| a.attribute("transform"))
                .collect::<Vec<_>>()
                .join(" ");
            let transform = if transform.is_empty() {
                "matrix(1 0 0 1 0 0)".to_string()
            } else {
                transform
            };
            shapes.push(Shape {
                d,
                fill,
                fill_opacity,
                stroke,
                stroke_width,
                stroke_opacity,
                line_cap,
                opacity,
                transform,
            });
        }
        if shapes.is_empty() {
            return Err(anyhow!("{} has no shape to draw", file.display()));
        }
        Ok(Drawing {
            view_x: view[0],
            view_y: view[1],
            view_width: view[2],
            view_height: view[3],
            shapes,
        })
    }

    /// The shapes in the drawing's own viewBox units.
    pub fn draw(&self) -> Svgr<'_> {
        self.shapes
            .iter()
            .map(|s| {
                fframes::svgr!(
                    <path d={s.d.as_str()} transform={s.transform.as_str()}
                        fill={s.fill.as_str()} fill-opacity={s.fill_opacity}
                        stroke={s.stroke.as_str()} stroke-width={s.stroke_width} stroke-opacity={s.stroke_opacity}
                        stroke-linecap={s.line_cap.as_str()} stroke-linejoin="round" opacity={s.opacity} />
                )
            })
            .collect()
    }
}
