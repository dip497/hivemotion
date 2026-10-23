//! The palette and type of the app, and the pieces every scene shares: labels, callouts, the
//! framed clip, the hexagon.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use fframes::{
    FFramesContext, FFramesSyncedVideoFrame, FontQuery, Frame, Svgr, SyncVideoFrameInput,
};

// The app's tokens (apps/desktop styles.css), turned to sRGB. Honey is the app's own warn colour,
// the one warm hue in a cool grey interface.
pub const BG: &str = "#0a0b0c";
pub const SURFACE: &str = "#111214";
pub const BONE: &str = "#e6e8ea";
pub const FG2: &str = "#bbbec1";
pub const GREY: &str = "#83868a";
pub const LINE: &str = "#252729";
pub const LINE2: &str = "#36383b";
pub const HONEY: &str = "#f1af57";
pub const AMBER: &str = "#c7843a";
pub const EMBER: &str = "#6e4a1f";
pub const INK: &str = "#0d0e10";
pub const OK: &str = "#88b898";
pub const ERR: &str = "#cc7867";

pub const SANS: &str = "Geist";
pub const MONO: &str = "JetBrains Mono";

/// Display weight: Geist Black, set tight.
pub const HEAVY: u16 = 900;

type WidthKey = (String, usize, u16, String);

static WIDTHS: LazyLock<Mutex<HashMap<WidthKey, f32>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Width of a string in pixels, measured once per font and text.
pub fn measure(
    frame: &mut Frame,
    ctx: &FFramesContext,
    family: &str,
    size: usize,
    weight: u16,
    text: &str,
) -> f32 {
    let key = (family.to_owned(), size, weight, text.to_owned());
    if let Some(w) = WIDTHS.lock().unwrap().get(&key) {
        return *w;
    }
    let query = FontQuery { family, size, weight, ..Default::default() };
    // `text_width` wants the text to outlive the context; each distinct string is kept once.
    let text: &'static str = Box::leak(text.to_owned().into_boxed_str());
    match frame.text_width(ctx, query, text) {
        Some(w) => {
            let w = w as f32;
            WIDTHS.lock().unwrap().insert(key, w);
            w
        }
        None => text.chars().count() as f32 * size as f32 * 0.6,
    }
}

/// Crop marks at the corners of a rectangle.
pub fn corners(x: f32, y: f32, w: f32, h: f32, arm: f32, color: &str, width: f32) -> Svgr<'static> {
    let d = format!(
        "M{x} {y1} V{y} H{x1} M{x2} {y} H{x3} V{y1} M{x3} {y2} V{y3} H{x2} M{x1} {y3} H{x} V{y2}",
        x1 = x + arm,
        x2 = x + w - arm,
        x3 = x + w,
        y1 = y + arm,
        y2 = y + h - arm,
        y3 = y + h,
    );
    fframes::svgr!(<path d={d} fill="none" stroke={color.to_owned()} stroke-width={width} />)
}

/// A small mono label in tracked capitals.
pub fn label(x: f32, y: f32, text: String, color: &str, size: f32, anchor: &'static str) -> Svgr<'static> {
    fframes::svgr!(
        <text x={x} y={y} font-family={MONO} font-weight="500" font-size={size} letter-spacing="2.5"
              fill={color.to_owned()} text-anchor={anchor}>{text}</text>
    )
}

/// An annotation: a dot on the target, an elbow line drawn on, then the label.
pub fn callout(tx: f32, ty: f32, lx: f32, ly: f32, text: String, color: &str, progress: f32) -> Svgr<'static> {
    if progress <= 0.0 {
        return Svgr::empty();
    }
    let p = progress.clamp(0.0, 1.0);
    let l1 = (lx - tx).abs() + 0.01;
    let l2 = (ly - ty).abs() + 0.01;
    let total = l1 + l2;
    let path = format!("M{tx} {ty} H{lx} V{ly}");
    let dash = format!("{} {}", total * p, total);
    let up = ly < ty;
    let text_y = if up { ly - 14.0 } else { ly + 30.0 };
    let anchor = if (lx - tx).abs() < 1.0 { "middle" } else if lx > tx { "end" } else { "start" };
    let text_o = prog01((p - 0.6) / 0.4);
    fframes::svgr!(
        <g>
            <circle cx={tx} cy={ty} r="5" fill={color.to_owned()} />
            <circle cx={tx} cy={ty} r="13" fill="none" stroke={color.to_owned()} stroke-width="1.5" opacity={p * 0.6} />
            <path d={path} fill="none" stroke={color.to_owned()} stroke-width="1.5" stroke-dasharray={dash} />
            <text x={lx} y={text_y} font-family={MONO} font-weight="700" font-size="20" letter-spacing="2.5"
                  fill={color.to_owned()} text-anchor={anchor} opacity={text_o}>{text}</text>
        </g>
    )
}

fn prog01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

/// Timecode `hh:mm:ss:ff` at 60 fps.
pub fn timecode(seconds: f32) -> String {
    let total = (seconds * 60.0).max(0.0) as u32;
    format!("00:{:02}:{:02}:{:02}", total / 3600, (total / 60) % 60, total % 60)
}

pub fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The outline of a pointy-top hexagon around (cx, cy), the shape of the app's mark.
pub fn hex_path(cx: f32, cy: f32, r: f32) -> String {
    let mut d = String::new();
    for i in 0..6 {
        let a = std::f32::consts::FRAC_PI_3 * i as f32 - std::f32::consts::FRAC_PI_2;
        let (x, y) = (cx + r * a.cos(), cy + r * a.sin());
        d.push_str(&format!("{}{x:.1} {y:.1} ", if i == 0 { "M" } else { "L" }));
    }
    d.push('Z');
    d
}

/// The Hivemind mark: one cell above, two below (the app's mark.svg, drawn at `size`).
pub fn mark(x: f32, y: f32, size: f32, top: &str, bottom: &str, bottom_opacity: f32) -> Svgr<'static> {
    let s = size / 24.0;
    let t = format!("translate({x} {y}) scale({s})");
    fframes::svgr!(
        <g transform={t}>
            <path fill={top.to_owned()} d="M11.14 1.70 Q12 1.20 12.86 1.70 L16.51 3.80 Q17.37 4.30 17.37 5.29 L17.37 9.51 Q17.37 10.50 16.51 11 L12.86 13.10 Q12 13.60 11.14 13.10 L7.49 11 Q6.63 10.50 6.63 9.51 L6.63 5.29 Q6.63 4.30 7.49 3.80 Z" />
            <g fill={bottom.to_owned()} opacity={bottom_opacity}>
                <path d="M5.60 11.30 Q6.46 10.80 7.32 11.30 L10.97 13.40 Q11.83 13.90 11.83 14.89 L11.83 19.11 Q11.83 20.10 10.97 20.60 L7.32 22.70 Q6.46 23.20 5.60 22.70 L1.95 20.60 Q1.09 20.10 1.09 19.11 L1.09 14.89 Q1.09 13.90 1.95 13.40 Z" />
                <path d="M16.68 11.30 Q17.54 10.80 18.40 11.30 L22.05 13.40 Q22.91 13.90 22.91 14.89 L22.91 19.11 Q22.91 20.10 22.05 20.60 L18.40 22.70 Q17.54 23.20 16.68 22.70 L13.03 20.60 Q12.17 20.10 12.17 19.11 L12.17 14.89 Q12.17 13.90 13.03 13.40 Z" />
            </g>
        </g>
    )
}

/// A clip of the real app, started `start_from` seconds into the scene, filling the box. A dark
/// box stands in before it starts or when the file is missing.
#[allow(clippy::too_many_arguments)]
pub fn clip<'a>(frame: &Frame, ctx: &FFramesContext<'a, '_>, name: &str, start_from: f32, x: f32, y: f32, w: f32, h: f32) -> Svgr<'a> {
    let input = SyncVideoFrameInput { start_from, looping: false, editor_fallback_image: None };
    match frame.get_synced_video_frame(ctx, name, &input).map(|f| f.into_image()) {
        Some(img) => fframes::svgr!(<image href={img.href()} x={x} y={y} width={w} height={h} preserveAspectRatio="xMidYMid slice" />),
        None => fframes::svgr!(<rect x={x} y={y} width={w} height={h} fill={SURFACE} />),
    }
}

/// A recording of the app in a frame: hairline border, crop marks and a caption row under it.
#[allow(clippy::too_many_arguments)]
pub fn framed_clip<'a>(
    frame: &Frame,
    ctx: &FFramesContext<'a, '_>,
    name: &str,
    start_from: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    caption: String,
) -> Svgr<'a> {
    let image = clip(frame, ctx, name, start_from, x, y, w, h);
    let tc = timecode((frame.seconds() - start_from).max(0.0));
    fframes::svgr!(
        <g>
            {image}
            <rect x={x} y={y} width={w} height={h} fill="none" stroke={LINE2} stroke-width="1.5" />
            {corners(x - 14.0, y - 14.0, w + 28.0, h + 28.0, 22.0, GREY, 1.5)}
            {label(x, y + h + 46.0, caption, GREY, 17.0, "start")}
            {label(x + w, y + h + 46.0, tc, GREY, 17.0, "end")}
        </g>
    )
}

/// A word that lands on a spring from `(dx, dy)` and leaves a honey trail while it moves.
pub struct Slam {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub size: f32,
    pub weight: u16,
    pub fill: &'static str,
    pub spacing: f32,
    pub anchor: &'static str,
}

impl Slam {
    pub fn new(x: f32, y: f32, text: impl Into<String>, size: f32, fill: &'static str) -> Self {
        Slam { x, y, text: text.into(), size, weight: HEAVY, fill, spacing: -size * 0.045, anchor: "start" }
    }

    pub fn anchor(mut self, anchor: &'static str) -> Self {
        self.anchor = anchor;
        self
    }

    /// `since` is beats since the entrance started.
    pub fn draw(&self, since: f32, dx: f32, dy: f32) -> Svgr<'static> {
        if since < 0.0 {
            return Svgr::empty();
        }
        let off = |t: f32| {
            let s = crate::beat::snap(t.max(0.0));
            ((1.0 - s) * dx, (1.0 - s) * dy)
        };
        let text = |ox: f32, oy: f32, fill: &str, opacity: f32| {
            let t = self.text.clone();
            fframes::svgr!(
                <text x={self.x + ox} y={self.y + oy} text-anchor={self.anchor} font-family={SANS} font-weight={self.weight}
                      font-size={self.size} letter-spacing={self.spacing} fill={fill.to_owned()} opacity={opacity}>{t}</text>
            )
        };
        let mut layers = Vec::new();
        for (lag, color, o) in [(0.16, EMBER, 0.4), (0.10, AMBER, 0.55), (0.05, HONEY, 0.7)] {
            let (ox, oy) = off(since - lag);
            let (cx, cy) = off(since);
            let moving = ((ox - cx).powi(2) + (oy - cy).powi(2)).sqrt();
            if moving > 2.0 && since > lag * 0.5 {
                layers.push(text(ox, oy, color, o * (moving / 40.0).min(1.0)));
            }
        }
        let (ox, oy) = off(since);
        layers.push(text(ox, oy, self.fill, crate::beat::prog(since, 0.0, 0.06)));
        fframes::svgr!(<g>{layers}</g>)
    }
}

/// A heading in two lines, the second in honey, each on its own spring.
pub fn two_line_title(lb: f32, x: f32, y: f32, size: f32, first: &str, second: &str) -> Svgr<'static> {
    let gap = size * 1.0;
    fframes::svgr!(
        <g>
            {Slam::new(x, y, first, size, BONE).draw(lb, 0.0, 70.0)}
            {Slam::new(x, y + gap, second, size, HONEY).draw(lb - 0.75, 0.0, 70.0)}
        </g>
    )
}
