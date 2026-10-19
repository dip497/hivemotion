//! Every number on screen, in one place. Each one is measured; facts.md gives the command.

/// The hivemind repository, origin/main at b712697 (2026-10-19).
pub const FIRST_COMMIT_HASH: &str = "66337e8";
pub const FIRST_COMMIT_DATE: &str = "2026-05-27";
pub const FIRST_COMMIT_SUBJECT: &str = "Initial OSS release of hivemind";
pub const DAYS: u64 = 145;
pub const COMMITS: u64 = 1_450;
pub const RELEASES: u64 = 76;
/// Lines in the git-tracked .ts, .tsx and .rs files (no .d.ts): 151,580 TypeScript + 41,857 Rust.
pub const LINES: u64 = 193_437;
/// Plugins in the HiveHub index (api/v1/index.json) on 2026-10-19.
pub const PLUGINS: u64 = 51;

/// This video, measured from its own first full render (the time scripts/render.sh prints).
pub const FRAMES: u64 = 4_394;
pub const RENDER_SECONDS: f32 = 204.5;
pub const RENDER_MACHINE: &str = "INTEL IRIS XE · SKIA ON VULKAN";
