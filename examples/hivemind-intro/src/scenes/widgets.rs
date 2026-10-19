//! Widgets and tools. An agent is asked for a CI light, writes it, puts it on the board, and the
//! light goes red, then green as the agent fixes the test. Then a log tail, a tool.

use fframes::{Duration, FFramesContext, Frame, Scene, Svgr};

use crate::beat::*;
use crate::shaders::ground;
use crate::ui::*;

beat_scene!(WidgetScene, 68.0, 84.0);
beat_scene!(ToolScene, 84.0, 88.0);

const BOX: (f32, f32, f32, f32) = (130.0, 210.0, 1060.0, 663.0);
/// Seconds of the cut (scripts/prep.sh) where the light shows, turns red and turns green.
const LIGHT_ON: f32 = 3.2;
const LIGHT_RED: f32 = 5.05;
const LIGHT_GREEN: f32 = 6.35;
/// Where the widget sits inside the box.
const WIDGET: (f32, f32, f32, f32) = (966.0, 216.0, 218.0, 96.0);

impl Scene for WidgetScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let t = frame.seconds();
        let (x, y, w, h) = BOX;
        let enter = expo_out(prog(lb, 0.0, 0.8));
        let (status, color, since) = if t >= LIGHT_GREEN {
            ("PASSED", OK, t - LIGHT_GREEN)
        } else if t >= LIGHT_RED {
            ("FAILED", ERR, t - LIGHT_RED)
        } else if t >= LIGHT_ON {
            ("RUNNING", HONEY, t - LIGHT_ON)
        } else {
            ("WRITING IT", GREY, t)
        };
        let s = snap(since / BEAT);
        let (wx, wy, ww, wh) = WIDGET;
        let ring = prog(t, LIGHT_ON, LIGHT_ON + 0.4);
        let per = 2.0 * (ww + wh);
        let note = match status {
            "PASSED" => "THE AGENT FIXED THE TEST",
            "FAILED" => "A TEST FAILED",
            "RUNNING" => "ON THE BOARD, WATCHING CI",
            _ => "WIDGET.JSON + WIDGET.JS",
        };
        let exit = expo_in(prog(lb, 15.7, 16.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {ground(&frame, 0.8, 0.3, 0.26, 0.1)}
                <g transform={format!("translate({} 0)", (1.0 - enter) * -160.0)} opacity={enter}>
                    {framed_clip(&frame, ctx, "widget.mp4", 0.0, x, y, w, h, "A WIDGET · RECORDED IN THE APP".to_owned())}
                    <rect x={wx - 6.0} y={wy - 6.0} width={ww + 12.0} height={wh + 12.0} fill="none" stroke={color} stroke-width="3"
                          stroke-dasharray={format!("{} {}", per * ring, per)} />
                </g>
                {Slam::new(1260.0, 300.0, "ASK FOR", 84.0, BONE).draw(lb - 0.2, 0.0, 60.0)}
                {Slam::new(1260.0, 386.0, "A WIDGET.", 84.0, HONEY).draw(lb - 0.7, 0.0, 60.0)}
                {label(1264.0, 560.0, "CI LIGHT".to_owned(), GREY, 20.0, "start")}
                <g transform={format!("translate(0 {})", (1.0 - s) * 40.0)} opacity={prog(since, 0.0, 0.05)}>
                    <text x="1258" y="660" font-family={SANS} font-weight={HEAVY} font-size="96" letter-spacing="-4" fill={color}>{status}</text>
                    {label(1264.0, 716.0, note.to_owned(), FG2, 19.0, "start")}
                </g>
            </g>
        )
    }
}

impl Scene for ToolScene {
    fn duration(&self) -> Duration<'_> {
        Self::frames()
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let lb = Self::lb(&frame);
        let enter = expo_out(prog(lb, 0.0, 0.6));
        let exit = expo_in(prog(lb, 3.75, 4.0));
        fframes::svgr!(
            <g opacity={1.0 - exit}>
                {ground(&frame, 0.2, 0.6, 0.26, 0.1)}
                {Slam::new(146.0, 420.0, "AND", 120.0, BONE).draw(lb, 0.0, 80.0)}
                {Slam::new(146.0, 540.0, "TOOLS.", 120.0, HONEY).draw(lb - 0.4, 0.0, 80.0)}
                {label(152.0, 620.0, "A LOG TAIL, LIVE".to_owned(), FG2, 20.0, "start")}
                <g transform={format!("translate({} 0)", (1.0 - enter) * 200.0)} opacity={enter}>
                    {framed_clip(&frame, ctx, "tool.mp4", 0.0, 640.0, 190.0, 1130.0, 706.0, "A TOOL · RECORDED IN THE APP".to_owned())}
                </g>
            </g>
        )
    }
}
