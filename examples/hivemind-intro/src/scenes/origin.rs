//! Where it started: the first commit, every commit since scrolling past, and the numbers.

use std::sync::LazyLock;

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::facts;
use crate::shaders::ground;
use crate::ui::*;

beat_scene!(OriginScene, 16.0, 32.0);

struct Commit {
    hash: String,
    date: String,
    subject: String,
}

/// `git log --reverse --format='%h|%ad|%s' --date=short` of hivemind, written by scripts/prep.sh.
static COMMITS: LazyLock<Vec<Commit>> = LazyLock::new(|| {
    let path = crate::takes_dir().join("commits.txt");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, '|');
            Some(Commit {
                hash: parts.next()?.to_owned(),
                date: parts.next()?.to_owned(),
                subject: parts.next()?.chars().filter(char::is_ascii).take(52).collect(),
            })
        })
        .collect()
});

const MONTHS: [&str; 12] = ["JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"];

fn month(date: &str) -> &'static str {
    let m = date.get(5..7).and_then(|m| m.parse::<usize>().ok()).unwrap_or(1);
    MONTHS[(m - 1).min(11)]
}

impl Scene for OriginScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, mut frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let well_x = 0.72 - prog(lb, 0.0, 16.0) * 0.3;
        let content = if lb < 4.0 {
            first_commit(&mut frame, ctx, lb)
        } else if lb < 8.0 {
            history(lb - 4.0)
        } else {
            stats(&mut frame, ctx, lb - 8.0)
        };
        fframes::svgr!(
            <g>
                {ground(&frame, well_x, 0.56, 0.36 + prog(lb, 12.0, 16.0) * 0.25, 0.12)}
                {content}
            </g>
        )
    }
}

/// Beats 0-4: `git log` typed, the date lands a part per beat, then the commit itself.
fn first_commit(frame: &mut Frame, ctx: &FFramesContext, lb: f32) -> Svgr<'static> {
    let command = "$ git log --reverse | head -1";
    let typed = ((lb / 0.9) * command.len() as f32) as usize;
    let command: String = command.chars().take(typed.min(command.len())).collect();
    let parts = ["MAY", "27", "2026"];
    let mut x = 150.0;
    let mut words = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        let at = 0.6 + i as f32 * 0.5;
        let s = snap(lb - at);
        let w = measure(frame, ctx, SANS, 250, HEAVY, part) - 11.0 * part.len() as f32;
        let fill = if i == 2 { HONEY } else { BONE };
        words.push(fframes::svgr!(
            <text x={x} y={620.0 + (1.0 - s) * 150.0} opacity={prog(lb, at, at + 0.08)} font-family={SANS} font-weight={HEAVY}
                  font-size="250" letter-spacing="-11" fill={fill}>{*part}</text>
        ));
        x += w + 60.0;
    }
    let meta = prog(lb, 2.3, 2.6);
    let underline = quart_out(prog(lb, 2.0, 2.8)) * (x - 210.0);
    let exit = expo_in(prog(lb, 3.7, 4.0));
    let hash_w = 7.0 * 34.0 * 0.6;
    fframes::svgr!(
        <g opacity={1.0 - exit} transform={format!("translate(0 {})", -exit * 100.0)}>
            <text x="150" y="250" font-family={MONO} font-weight="500" font-size="30" fill={GREY}>{command}</text>
            {words}
            <rect x="150" y="676" width={underline.max(0.5)} height="10" fill={HONEY} />
            <g opacity={meta}>
                <text x="150" y="770" font-family={MONO} font-weight="700" font-size="34" fill={HONEY}>{facts::FIRST_COMMIT_HASH}</text>
                <text x={150.0 + hash_w + 30.0} y="770" font-family={MONO} font-weight="500" font-size="34" fill={BONE}>{format!("“{}”", facts::FIRST_COMMIT_SUBJECT)}</text>
            </g>
            {callout(150.0 + hash_w / 2.0, 786.0, 150.0 + hash_w / 2.0, 900.0, "FIRST COMMIT".to_owned(), HONEY, prog(lb, 2.6, 3.3))}
        </g>
    )
}

/// Beats 4-8: every commit scrolls past, faster and faster, and the month flips.
fn history(lb: f32) -> Svgr<'static> {
    let commits = &*COMMITS;
    let n = commits.len();
    if n == 0 {
        return Svgr::empty();
    }
    let enter = expo_out(prog(lb, 0.0, 0.4));
    let exit = expo_in(prog(lb, 3.7, 4.0));
    let pos = cubic_in_out(prog(lb, 0.2, 3.6)) * (n - 1) as f32;
    let current = (pos.round() as usize).min(n - 1);
    let line_h = 44.0;
    let center_y = 560.0;
    let first = pos.floor() as i32 - 11;
    let rows: Vec<Svgr> = (first..first + 24)
        .filter(|i| *i >= 0 && (*i as usize) < n)
        .map(|i| {
            let c = &commits[i as usize];
            let y = center_y + (i as f32 - pos) * line_h;
            let o = (1.0 - ((y - center_y) / 470.0).abs()).clamp(0.0, 1.0).powf(1.5);
            let is_current = i as usize == current;
            fframes::svgr!(
                <g opacity={o}>
                    <text x="150" y={y} font-family={MONO} font-weight="700" font-size="25" fill={if is_current { HONEY } else { EMBER }}>{c.hash.clone()}</text>
                    <text x="290" y={y} font-family={MONO} font-weight="500" font-size="25" fill={GREY}>{c.date.clone()}</text>
                    <text x="482" y={y} font-family={MONO} font-weight="500" font-size="25" fill={if is_current { BONE } else { "#7d8186" }}>{c.subject.clone()}</text>
                </g>
            )
        })
        .collect();
    let c = &commits[current];
    let count = format!("#{:04} / {}", current + 1, thousands(n as u64));
    fframes::svgr!(
        <g opacity={enter * (1.0 - exit)} transform={format!("translate(0 {})", (1.0 - enter) * 80.0 - exit * 80.0)}>
            <rect x="130" y={center_y - 33.0} width="1000" height="46" fill={HONEY} fill-opacity="0.12" />
            <rect x="130" y={center_y - 33.0} width="4" height="46" fill={HONEY} />
            {rows}
            <rect x="1180" y="300" width="620" height="460" fill={BG} fill-opacity="0.85" />
            <text x="1770" y="600" text-anchor="end" font-family={SANS} font-weight={HEAVY} font-size="300" letter-spacing="-13" fill={HONEY}>{month(&c.date)}</text>
            <text x="1770" y="680" text-anchor="end" font-family={MONO} font-weight="500" font-size="36" fill={BONE}>{count}</text>
            <text x="1770" y="730" text-anchor="end" font-family={MONO} font-weight="500" font-size="26" fill={GREY}>{c.date.clone()}</text>
        </g>
    )
}

/// Beats 8-16: the numbers, one per bar, counting up as they land.
fn stats(frame: &mut Frame, ctx: &FFramesContext, lb: f32) -> Svgr<'static> {
    let items: [(u64, &str, String); 2] = [
        (
            facts::DAYS,
            "DAYS",
            format!("27 MAY TO 19 OCT 2026  ·  {} COMMITS  ·  {} RELEASES", thousands(facts::COMMITS), facts::RELEASES),
        ),
        (facts::LINES, "LINES", "OF TYPESCRIPT AND RUST, IN THE OPEN".to_owned()),
    ];
    let bar = ((lb / 4.0).floor() as usize).min(1);
    let local = lb - bar as f32 * 4.0;
    let (value, unit, caption) = items[bar].clone();
    let enter = expo_out(prog(local, 0.0, 0.7));
    let exit = if bar == 1 { expo_in(prog(local, 3.75, 4.0)) } else { expo_in(prog(local, 3.65, 4.0)) };
    let x = 150.0 + (1.0 - enter) * 260.0 - exit * 300.0;
    let num = thousands(value);
    let num_w = measure(frame, ctx, SANS, 380, HEAVY, &num) - 17.0 * num.len() as f32;
    let unit_s = snap(local - 0.5);
    let cap = prog(local, 1.0, 1.4);
    let shown = thousands((value as f32 * expo_out(prog(local, 0.0, 1.3))).round() as u64);
    let unit_w = measure(frame, ctx, SANS, 120, HEAVY, unit) - 5.0 * unit.len() as f32;
    // the unit sits beside the number when it fits, under it otherwise
    let (unit_x, unit_y, cap_y) = if 150.0 + num_w + 40.0 + unit_w < 1770.0 { (num_w + 40.0, 0.0, 0.0) } else { (8.0, 140.0, 140.0) };
    fframes::svgr!(
        <g opacity={1.0 - exit}>
            <g transform={format!("translate({x} 0)")}>
                <text x="0" y="620" font-family={SANS} font-weight={HEAVY} font-size="380" letter-spacing="-17" fill={BONE}>{shown}</text>
                <g transform={format!("translate({} {})", unit_x, unit_y + (1.0 - unit_s) * 60.0)} opacity={unit_s.min(1.0)}>
                    <text x="0" y="620" font-family={SANS} font-weight={HEAVY} font-size="120" letter-spacing="-5" fill={HONEY}>{unit}</text>
                </g>
                <rect x="6" y={690.0 + cap_y} width={(cap * 60.0).max(0.5)} height="4" fill={HONEY} />
                <text x="6" y={752.0 + cap_y} font-family={MONO} font-weight="500" font-size="30" letter-spacing="3" fill={BONE} opacity={cap}>{caption}</text>
            </g>
        </g>
    )
}
