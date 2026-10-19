//! The one fframes `Video` that draws any film: it reads the resolved timeline at run time, so a
//! changed film.yaml needs no recompile.

use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::path::PathBuf;

use fframes::media::{FrameConvertOptions, ResizeVideoFrame};
use fframes::{
    AudioMap, Color, Duration, FFramesContext, FFramesSyncedVideoFrame, Frame, Svgr,
    SyncVideoFrameInput,
};
use hivemotion_spec::film::{Aspect, TransitionKind};
use hivemotion_spec::timeline::{
    caption_motion, AspectPlan, Picture, TimedCaption, TimedGraphic, TimedShot, Timeline,
};

use crate::drawing::Drawing;
use crate::fonts::FontMetrics;

pub(crate) fn seg(t: f64, a: f64, b: f64) -> f64 {
    if b <= a {
        return if t >= b { 1.0 } else { 0.0 };
    }
    ((t - a) / (b - a)).clamp(0.0, 1.0)
}
fn lerp(a: f64, b: f64, k: f64) -> f64 {
    a + (b - a) * k
}
fn out_cubic(x: f64) -> f64 {
    1.0 - (1.0 - x).powi(3)
}
fn out_quint(x: f64) -> f64 {
    1.0 - (1.0 - x).powi(5)
}
fn in_cubic(x: f64) -> f64 {
    x * x * x
}

/// cubic-bezier(.32,.72,0,1): a fast start that settles softly, used by the hexagon moves.
fn settle(x: f64) -> f64 {
    let (x1, y1, x2, y2) = (0.32, 0.72, 0.0, 1.0);
    let curve = |t: f64, a: f64, b: f64| {
        3.0 * (1.0 - t) * (1.0 - t) * t * a + 3.0 * (1.0 - t) * t * t * b + t * t * t
    };
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if curve(mid, x1, x2) < x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    curve((lo + hi) / 2.0, y1, y2)
}

fn hexagon(cx: f64, cy: f64, r: f64) -> String {
    (0..6)
        .map(|i| {
            let a = i as f64 * PI / 3.0 + PI / 6.0;
            format!("{:.2},{:.2}", cx + a.cos() * r, cy + a.sin() * r)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone)]
pub struct Look {
    pub background: String,
    pub text: String,
    pub text_dim: String,
    pub text_faint: String,
    pub sans: String,
    pub mono: String,
    pub caption_weight: u16,
    pub caption_tracking: f64,
    pub caption_line_height: f64,
}

/// One caption laid out for this aspect.
#[derive(Debug, Clone)]
pub struct CaptionLayout {
    pub caption: TimedCaption,
    pub size: f64,
    pub left: f64,
    /// Top of each line box.
    pub tops: Vec<f64>,
    pub baselines: Vec<f64>,
    pub line_box: f64,
}

#[derive(Debug, Clone)]
pub struct EndCardLayout {
    pub start: f64,
    pub mark_top: f64,
    pub mark_height: f64,
    pub title: String,
    pub title_size: f64,
    pub title_baseline: f64,
    pub line: String,
    pub line_size: f64,
    pub line_baseline: f64,
    pub command: String,
    pub pill_size: f64,
    pub pill_top: f64,
    pub pill_width: f64,
    pub pill_height: f64,
    pub pill_baseline: f64,
    pub prompt_width: f64,
}

pub struct FilmVideo<'p, const W: usize, const H: usize, const FPS: usize> {
    pub timeline: &'p Timeline,
    pub plan: &'p AspectPlan,
    pub look: Look,
    pub mark: Option<Drawing>,
    /// The drawings of every graphic layer, by file.
    pub drawings: HashMap<PathBuf, Drawing>,
    /// Shots and insets with a prepared clip, by id.
    pub clips: HashSet<String>,
    pub captions: Vec<CaptionLayout>,
    pub end_card: Option<EndCardLayout>,
    pub frames: usize,
}

/// The faces text is measured with: captions, the end card's title and line, and its command.
pub struct TextFonts {
    pub caption: FontMetrics,
    pub title: FontMetrics,
    pub line: FontMetrics,
    pub mono: FontMetrics,
}

/// Lays out captions and the end card for one aspect, shrinking any text that would not fit.
/// Returns the layouts and a note for every shrink.
pub fn layout(
    timeline: &Timeline,
    plan: &AspectPlan,
    look: &Look,
    fonts: &TextFonts,
    brand_name: &str,
) -> (Vec<CaptionLayout>, Option<EndCardLayout>, Vec<String>) {
    let TextFonts {
        caption: caption_font,
        title: title_font,
        line: line_font,
        mono: mono_font,
    } = fonts;
    let (w, h) = (plan.width as f64, plan.height as f64);
    let mut notes = Vec::new();
    let place = &plan.caption;
    let lh = look.caption_line_height;
    let mut captions = Vec::new();
    for c in plan.captions(&timeline.captions) {
        let max = w - 2.0 * place.left;
        let widest = c
            .lines
            .iter()
            .map(|l| caption_font.width(l, place.size, look.caption_tracking))
            .fold(0.0, f64::max);
        let size = if widest > max {
            let s = (place.size * max / widest).floor();
            notes.push(format!(
                "{} caption \"{}\" shrunk from {} to {} px to fit",
                plan.aspect, c.lines[0], place.size, s
            ));
            s
        } else {
            place.size
        };
        let n = c.lines.len() as f64;
        let first_top = match (place.top, place.bottom) {
            (Some(top), _) => top,
            (None, Some(bottom)) => h - bottom - size - (n - 1.0) * size * lh,
            (None, None) => h * 0.7,
        };
        let tops: Vec<f64> = (0..c.lines.len())
            .map(|i| first_top + i as f64 * size * lh)
            .collect();
        let baselines = tops
            .iter()
            .map(|t| caption_font.baseline(*t, size, lh))
            .collect();
        captions.push(CaptionLayout {
            caption: c,
            size,
            left: place.left,
            tops,
            baselines,
            line_box: size * lh,
        });
    }

    let end_card = timeline.end_card.as_ref().and_then(|card| {
        let start = plan.render_time(card.start)?;
        let vertical = h > w;
        let s = h / if vertical { 1920.0 } else { 1080.0 };
        let margin = 64.0 * s;
        let fit = |font: &FontMetrics,
                   text: &str,
                   size: f64,
                   tracking: f64,
                   extra: f64,
                   notes: &mut Vec<String>| {
            let width = font.width(text, size, tracking) + extra * size;
            if width > w - 2.0 * margin {
                let fitted = (size * (w - 2.0 * margin) / width).floor();
                notes.push(format!(
                    "{} end card \"{text}\" shrunk from {size} to {fitted} px to fit",
                    plan.aspect
                ));
                fitted
            } else {
                size
            }
        };
        let mark_height = if vertical { 200.0 } else { 150.0 } * s;
        let title_size = fit(
            title_font,
            brand_name,
            if vertical { 150.0 } else { 132.0 } * s,
            -0.045,
            0.0,
            &mut notes,
        );
        let line_size = fit(
            line_font,
            &card.line,
            if vertical { 40.0 } else { 38.0 } * s,
            -0.02,
            0.0,
            &mut notes,
        );
        let prompt = "$ ";
        let pill_size = fit(
            mono_font,
            &format!("{prompt}{}", card.command),
            if vertical { 27.0 } else { 30.0 } * s,
            0.0,
            2.1,
            &mut notes,
        );
        let gap_after_mark = if vertical { 70.0 } else { 44.0 } * s;
        let pill_gap = if vertical { 150.0 } else { 84.0 } * s;
        let normal = |f: &FontMetrics, size: f64| (f.ascent + f.descent) * size;
        let pill_height = normal(mono_font, pill_size) + 2.0 * 0.62 * pill_size;
        let total = mark_height + gap_after_mark + title_size * 1.12 + pill_gap + pill_height;
        let mark_top = (h - total) / 2.0;
        let title_top = mark_top + mark_height + gap_after_mark;
        let line_top = title_top + title_size * 1.12;
        let pill_top = line_top + pill_gap;
        let pill_text = mono_font.width(&format!("{prompt}{}", card.command), pill_size, 0.0);
        Some(EndCardLayout {
            start,
            mark_top,
            mark_height,
            title: brand_name.to_string(),
            title_size,
            title_baseline: title_top + title_font.ascent * title_size,
            line: card.line.clone(),
            line_size,
            line_baseline: line_top + line_font.ascent * line_size,
            command: card.command.clone(),
            pill_size,
            pill_top,
            pill_width: pill_text + 2.0 * 1.05 * pill_size,
            pill_height,
            pill_baseline: pill_top + 0.62 * pill_size + mono_font.ascent * pill_size,
            prompt_width: mono_font.width(prompt, pill_size, 0.0),
        })
    });
    (captions, end_card, notes)
}

/// The scale and top-left of a clip of `cw`×`ch` pixels at progress `u` of its pan and zoom:
/// never larger than the clip's own pixels unless that is the only way to cover the frame.
pub fn place_clip(
    shot: &TimedShot,
    aspect: Aspect,
    u: f64,
    cw: f64,
    ch: f64,
    w: f64,
    h: f64,
) -> (f64, f64, f64) {
    let cover = (w / cw).max(h / ch);
    let (zoom, fx, fy) = match shot.pan_zoom.get(&aspect) {
        Some(pz) => (
            lerp(pz.start.zoom, pz.end.zoom, u),
            lerp(pz.start.x, pz.end.x, u),
            lerp(pz.start.y, pz.end.y, u),
        ),
        None => (cover, 0.5, 0.5),
    };
    let k = zoom.max(cover).min(1.0f64.max(cover));
    let x = (w / 2.0 - fx * cw * k).clamp(w - cw * k, 0.0);
    let y = (h / 2.0 - fy * ch * k).clamp(h - ch * k, 0.0);
    (k, x, y)
}

enum ShotClip {
    None,
    Hexagon(f64),
    Fade(f64),
}

impl<'p, const W: usize, const H: usize, const FPS: usize> FilmVideo<'p, W, H, FPS> {
    fn w(&self) -> f64 {
        W as f64
    }
    fn h(&self) -> f64 {
        H as f64
    }

    fn slate<'a>(&'a self, label: String, x: f64, y: f64, w: f64, h: f64) -> Svgr<'a> {
        let size = 28.0 * (self.h().min(self.w()) / 1080.0);
        fframes::svgr!(
            <g>
                <rect x={x} y={y} width={w} height={h} fill="url(#slate)" />
                <text x={x + w / 2.0} y={y + h / 2.0 + size * 0.35} text-anchor="middle"
                    font-family={self.look.mono.as_str()} font-size={size} font-weight="500"
                    fill={self.look.text_faint.as_str()}>{label}</text>
            </g>
        )
    }

    /// A graphic `t` seconds into its shot: its background, then each layer at its pose.
    fn graphic<'a>(&'a self, g: &TimedGraphic, t: f64) -> Svgr<'a> {
        let (w, h) = (self.w(), self.h());
        let background = g
            .background
            .clone()
            .unwrap_or_else(|| self.look.background.clone());
        let layers: Vec<Svgr<'a>> = g
            .layers
            .iter()
            .filter_map(|layer| {
                let d = self.drawings.get(&layer.svg)?;
                let p = layer.pose_at(t);
                let size = layer.width * w.min(h) / d.view_width * p.scale;
                // squash keeps the area: flatter is wider
                let (sx, sy) = (size / p.squash.sqrt(), size * p.squash.sqrt());
                let place = format!(
                    "translate({:.3},{:.3}) rotate({:.3}) scale({:.5},{:.5}) translate({:.3},{:.3})",
                    p.x * w,
                    p.y * h,
                    p.rotate,
                    sx,
                    sy,
                    -d.view_x - d.view_width / 2.0,
                    -d.view_y - d.view_height / 2.0
                );
                Some(fframes::svgr!(<g transform={place} opacity={p.opacity}>{d.draw()}</g>))
            })
            .collect();
        fframes::svgr!(
            <g>
                <rect x="0" y="0" width={w} height={h} fill={background} />
                {layers}
            </g>
        )
    }

    fn take_label(shot_clip: &std::path::Path) -> String {
        let stem = shot_clip
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        format!("take pending: {stem}")
    }

    /// The picture of one shot at film time `t`.
    fn shot_picture<'a>(
        &'a self,
        frame: &Frame,
        ctx: &FFramesContext<'a, '_>,
        shot: &TimedShot,
        t: f64,
        delta: f64,
    ) -> Svgr<'a> {
        let (w, h) = (self.w(), self.h());
        let clip = match &shot.picture {
            Picture::Graphic(g) => return self.graphic(g, t - shot.start),
            Picture::Clip(clip) => clip,
        };
        if !self.clips.contains(&shot.id) {
            return self.slate(Self::take_label(clip), 0.0, 0.0, w, h);
        }
        let input = SyncVideoFrameInput {
            start_from: (shot.start - delta + 1e-4).max(0.0) as f32,
            looping: false,
            editor_fallback_image: None,
        };
        let Some(video) = frame.get_synced_video_frame(ctx, format!("{}.mp4", shot.id), &input)
        else {
            return Svgr::empty();
        };
        let (cw, ch) = (video.width() as f64, video.height() as f64);
        let u = shot.ease.apply(seg(t, shot.start, shot.end));
        let (k, x, y) = place_clip(shot, self.plan.aspect, u, cw, ch, w, h);
        let (dw, dh) = ((cw * k).round().max(1.0), (ch * k).round().max(1.0));
        let image = video
            .into_resized_image(&FrameConvertOptions {
                resize: ResizeVideoFrame {
                    width: dw as u32,
                    height: dh as u32,
                },
            })
            .unwrap_or_else(|| video.into_image());
        fframes::svgr!(<image href={image.href()} x={x} y={y} width={dw} height={dh} />)
    }

    fn shots<'a>(
        &'a self,
        frame: &Frame,
        ctx: &FFramesContext<'a, '_>,
        t: f64,
        delta: f64,
    ) -> Vec<Svgr<'a>> {
        let shots = &self.timeline.shots;
        let (w, h) = (self.w(), self.h());
        let reach = (w * w + h * h).sqrt() * 0.62;
        // (shot index, how it is cut out), bottom first
        let mut order: Vec<(usize, ShotClip)> = Vec::new();
        for (i, shot) in shots.iter().enumerate() {
            if t < shot.start || t >= shot.end + shot.tail {
                continue;
            }
            match shot.transition {
                Some((kind, length)) if t < shot.start + length => {
                    let u = seg(t, shot.start, shot.start + length);
                    match kind {
                        TransitionKind::HexWipe => {
                            order.push((i, ShotClip::Hexagon((reach * settle(u)).max(1.0))))
                        }
                        TransitionKind::Cross => order.push((i, ShotClip::Fade(u))),
                        TransitionKind::HexIris => {
                            // the new shot goes under the one before, which closes into a hexagon
                            let r = lerp(reach, h * 0.05, settle(u)).max(1.0);
                            match order.iter().position(|(j, _)| *j + 1 == i) {
                                Some(p) => {
                                    order[p].1 = ShotClip::Hexagon(r);
                                    order.insert(p, (i, ShotClip::None));
                                }
                                None => order.push((i, ShotClip::None)),
                            }
                        }
                    }
                }
                _ => order.push((i, ShotClip::None)),
            }
        }
        order
            .into_iter()
            .map(|(i, clip)| {
                let picture = self.shot_picture(frame, ctx, &shots[i], t, delta);
                match clip {
                    ShotClip::None => picture,
                    ShotClip::Fade(o) => fframes::svgr!(<g opacity={o}>{picture}</g>),
                    ShotClip::Hexagon(r) => {
                        let id = format!("shot-cut-{i}");
                        let url = format!("url(#{id})");
                        fframes::svgr!(
                            <g>
                                <defs><clipPath id={id}><polygon points={hexagon(w / 2.0, h / 2.0, r)} /></clipPath></defs>
                                <g clip-path={url}>{picture}</g>
                            </g>
                        )
                    }
                }
            })
            .collect()
    }

    fn insets<'a>(
        &'a self,
        frame: &Frame,
        ctx: &FFramesContext<'a, '_>,
        t: f64,
        delta: f64,
    ) -> Vec<Svgr<'a>> {
        let (w, h) = (self.w(), self.h());
        let vertical = h > w;
        let mut out = Vec::new();
        for (i, inset) in self.timeline.insets.iter().enumerate() {
            if t < inset.start - 0.01 || t > inset.end {
                continue;
            }
            let kin = out_quint(seg(t, inset.start, inset.start + 0.6));
            let kout = in_cubic(seg(t, inset.end - 0.5, inset.end));
            let video = if self.clips.contains(&inset.id) {
                let input = SyncVideoFrameInput {
                    start_from: (inset.start - delta + 1e-4).max(0.0) as f32,
                    looping: false,
                    editor_fallback_image: None,
                };
                frame.get_synced_video_frame(ctx, format!("{}.mp4", inset.id), &input)
            } else {
                None
            };
            let (cw, ch) = video
                .as_ref()
                .map(|v| (v.width() as f64, v.height() as f64))
                .unwrap_or((1170.0, 2532.0));
            let bh = h * if vertical {
                1180.0 / 1920.0
            } else {
                860.0 / 1080.0
            };
            let bw = bh * cw / ch;
            let x = if vertical {
                (w - bw) / 2.0
            } else {
                w - bw - 150.0 * w / 1920.0
            };
            let y = (h - bh) / 2.0 + (1.0 - kin) * 120.0 + kout * 80.0;
            let radius = 44.0 * bh / 860.0;
            let picture = match &video {
                Some(v) => {
                    let k = (bw / cw).max(bh / ch);
                    let (dw, dh) = ((cw * k).round().max(1.0), (ch * k).round().max(1.0));
                    let image = v
                        .into_resized_image(&FrameConvertOptions {
                            resize: ResizeVideoFrame {
                                width: dw as u32,
                                height: dh as u32,
                            },
                        })
                        .unwrap_or_else(|| v.into_image());
                    fframes::svgr!(<image href={image.href()} x={(bw - dw) / 2.0} y={(bh - dh) / 2.0} width={dw} height={dh} />)
                }
                None => self.slate(Self::take_label(&inset.clip), 0.0, 0.0, bw, bh),
            };
            let id = format!("inset-cut-{i}");
            let url = format!("url(#{id})");
            let place = format!("translate({x:.2},{y:.2})");
            out.push(fframes::svgr!(
                <g transform={place} opacity={kin * (1.0 - kout)}>
                    <defs><clipPath id={id}><rect width={bw} height={bh} rx={radius} /></clipPath></defs>
                    <g clip-path={url}>
                        <rect width={bw} height={bh} fill="#000000" />
                        {picture}
                    </g>
                    <rect width={bw} height={bh} rx={radius} fill="none" stroke="#ffffff" stroke-opacity="0.16" stroke-width="2" />
                </g>
            ));
        }
        out
    }

    fn captions<'a>(&'a self, tr: f64) -> Vec<Svgr<'a>> {
        use caption_motion::*;
        let mut out = Vec::new();
        for (ci, c) in self.captions.iter().enumerate() {
            let (a, b) = (c.caption.start, c.caption.end);
            if tr < a - 0.01 || tr > b + 0.01 {
                continue;
            }
            for (li, line) in c.caption.lines.iter().enumerate() {
                let l = li as f64;
                let kin = out_cubic(seg(
                    tr,
                    a + l * LINE_STAGGER,
                    a + LINE_IN + l * LINE_STAGGER,
                ));
                let kout = in_cubic(seg(tr, b - OUT + l * OUT_STAGGER, b));
                let dy = ((1.0 - kin) - kout) * 1.05 * c.line_box;
                let id = format!("caption-{ci}-{li}");
                let url = format!("url(#{id})");
                out.push(fframes::svgr!(
                    <g>
                        <defs><clipPath id={id}><rect x="0" y={c.tops[li]} width={self.w()} height={c.line_box + 0.1 * c.size} /></clipPath></defs>
                        <g clip-path={url}>
                            <text x={c.left} y={c.baselines[li] + dy} font-family={self.look.sans.as_str()}
                                font-size={c.size} font-weight={self.look.caption_weight}
                                letter-spacing={self.look.caption_tracking * c.size}
                                fill={self.look.text.as_str()}>{line.as_str()}</text>
                        </g>
                    </g>
                ));
            }
        }
        out
    }

    fn end_card<'a>(&'a self, tr: f64) -> Svgr<'a> {
        let Some(card) = &self.end_card else {
            return Svgr::empty();
        };
        let s = card.start;
        if tr < s {
            return Svgr::empty();
        }
        let w = self.w();
        let shown = |d: f64| out_cubic(seg(tr, s + 0.30 + d, s + 0.80 + d));
        let mark = match &self.mark {
            Some(m) => {
                let scale = card.mark_height / m.view_height;
                let x = w / 2.0 - m.view_width * scale / 2.0 - m.view_x * scale;
                let y = card.mark_top - m.view_y * scale;
                let place = format!("translate({x:.2},{y:.2}) scale({scale:.5})");
                fframes::svgr!(<g transform={place} opacity={out_cubic(seg(tr, s, s + 0.45))}>{m.draw()}</g>)
            }
            None => Svgr::empty(),
        };
        let (k0, k1, k2) = (shown(0.0), shown(0.12), shown(0.24));
        let pill_x = w / 2.0 - card.pill_width / 2.0;
        let text_x = pill_x + 1.05 * card.pill_size;
        fframes::svgr!(
            <g>
                {mark}
                <text x={w / 2.0} y={card.title_baseline + (1.0 - k0) * 24.0} text-anchor="middle"
                    font-family={self.look.sans.as_str()} font-size={card.title_size} font-weight="700"
                    letter-spacing={-0.045 * card.title_size} fill={self.look.text.as_str()} opacity={k0}>{card.title.as_str()}</text>
                <text x={w / 2.0} y={card.line_baseline + (1.0 - k1) * 18.0} text-anchor="middle"
                    font-family={self.look.sans.as_str()} font-size={card.line_size} font-weight="500"
                    letter-spacing={-0.02 * card.line_size} fill={self.look.text_dim.as_str()} opacity={k1}>{card.line.as_str()}</text>
                <g opacity={k2} transform={format!("translate(0,{:.2})", (1.0 - k2) * 18.0)}>
                    <rect x={pill_x} y={card.pill_top} width={card.pill_width} height={card.pill_height} rx="16"
                        fill="#121417" fill-opacity="0.9" stroke="#ffffff" stroke-opacity="0.16" stroke-width="1" />
                    <text x={text_x} y={card.pill_baseline} font-family={self.look.mono.as_str()}
                        font-size={card.pill_size} font-weight="500" fill={self.look.text_faint.as_str()}>"$"</text>
                    <text x={text_x + card.prompt_width} y={card.pill_baseline} font-family={self.look.mono.as_str()}
                        font-size={card.pill_size} font-weight="500" fill="#e8ebee">{card.command.as_str()}</text>
                </g>
            </g>
        )
    }

    fn definitions<'a>(&'a self) -> Svgr<'a> {
        let vertical = H > W;
        let scrim = if vertical {
            fframes::svgr!(
                <linearGradient id="scrim" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0" stop-color="#020305" stop-opacity="0.85" />
                    <stop offset="0.24" stop-color="#020305" stop-opacity="0.5" />
                    <stop offset="0.40" stop-color="#020305" stop-opacity="0" />
                </linearGradient>
            )
        } else {
            fframes::svgr!(
                <linearGradient id="scrim" x1="0" y1="1" x2="0" y2="0">
                    <stop offset="0" stop-color="#020305" stop-opacity="0.86" />
                    <stop offset="0.30" stop-color="#020305" stop-opacity="0.55" />
                    <stop offset="0.58" stop-color="#020305" stop-opacity="0" />
                </linearGradient>
            )
        };
        fframes::svgr!(
            <defs>
                {scrim}
                <radialGradient id="vignette" cx="0.5" cy="0.46" r="0.72">
                    <stop offset="0.52" stop-color="#000000" stop-opacity="0" />
                    <stop offset="1" stop-color="#000000" stop-opacity="0.62" />
                </radialGradient>
                <radialGradient id="slate" cx="0.5" cy="0.45" r="0.75">
                    <stop offset="0" stop-color="#1a1f27" />
                    <stop offset="1" stop-color="#07080a" />
                </radialGradient>
            </defs>
        )
    }
}

impl<'p, const W: usize, const H: usize, const FPS: usize> fframes::Video
    for FilmVideo<'p, W, H, FPS>
{
    const FPS: usize = FPS;
    const WIDTH: usize = W;
    const HEIGHT: usize = H;
    const BACKGROUND_COLOR: Color = Color::BLACK;

    fn duration(&self) -> Duration<'_> {
        Duration::Frames(self.frames)
    }

    fn audio(&self) -> AudioMap<'_> {
        AudioMap::none()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let (w, h) = (self.w(), self.h());
        let tr = frame.index as f64 / FPS as f64;
        let t = self.plan.film_time(tr);
        let delta = t - tr;
        let shots = self.shots(&frame, ctx, t, delta);
        let insets = self.insets(&frame, ctx, t, delta);
        let card_start = self.end_card.as_ref().map(|c| c.start);
        let (dim, scrim) = match card_start {
            Some(s) => (
                0.85 * out_cubic(seg(tr, s + 0.15, s + 0.65)),
                0.7 * (1.0 - seg(tr, s - 0.25, s)),
            ),
            None => (0.0, 0.7),
        };
        let captions = self.captions(tr);
        let card = self.end_card(tr);
        let view = format!("0 0 {W} {H}");
        fframes::svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" viewBox={view} width={W} height={H}>
                {self.definitions()}
                <rect width={w} height={h} fill={self.look.background.as_str()} />
                {shots}
                {insets}
                <rect width={w} height={h} fill="#000000" opacity={dim} />
                <rect width={w} height={h} fill="url(#scrim)" opacity={scrim} />
                <rect width={w} height={h} fill="url(#vignette)" />
                {captions}
                {card}
            </svg>
        )
    }
}
