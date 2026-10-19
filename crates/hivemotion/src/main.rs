//! `hivemotion`: launch films from film.yaml. Every command takes `--json` and then prints one
//! JSON object with `ok`; it exits 1 when `ok` is false.

mod new;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use hivemotion_audio::Gate;
use hivemotion_deliver::PresetName;
use hivemotion_spec::{LoadError, Project, TimeRef};
use serde_json::{json, Value};

#[derive(Parser)]
#[command(
    name = "hivemotion",
    version,
    about = "Launch films from a film.yaml, rendered with fframes"
)]
struct Cli {
    /// Print one JSON object instead of text.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start a new film folder with a film.yaml, a brand.yaml and fonts.
    New {
        dir: PathBuf,
        /// The product's name.
        #[arg(long, default_value = "Product")]
        name: String,
    },
    /// Check film.yaml and brand.yaml; every problem names its field.
    Validate {
        #[arg(default_value = "film.yaml")]
        film: PathBuf,
        /// Also write film.schema.json and brand.schema.json into this folder.
        #[arg(long)]
        write_schema: Option<PathBuf>,
    },
    /// Cut every take to its shot (start, speed, length) with ffmpeg.
    Prep {
        #[arg(default_value = "film.yaml")]
        film: PathBuf,
    },
    /// Render the picture of each aspect to ProRes 4444, or single stills.
    Render {
        #[arg(default_value = "film.yaml")]
        film: PathBuf,
        /// Only this aspect: 16:9, 9:16 or 4:5.
        #[arg(long)]
        aspect: Option<String>,
        /// Render only stills at these times (seconds or beats, comma separated) into build/stills.
        #[arg(long, value_delimiter = ',')]
        stills: Vec<String>,
    },
    /// Lay out and master the sound, gate its loudness, and put it into the master.
    Master {
        #[arg(default_value = "film.yaml")]
        film: PathBuf,
        #[arg(long)]
        aspect: Option<String>,
        /// Report loudness without failing on it.
        #[arg(long)]
        no_gate: bool,
    },
    /// Make posters, cutdowns and platform files from the masters.
    Deliver {
        #[arg(default_value = "film.yaml")]
        film: PathBuf,
        /// x, youtube, shorts, linkedin, readme-loop, producthunt (default: all that apply).
        #[arg(long, value_delimiter = ',')]
        preset: Vec<String>,
    },
    /// Stills at every beat and mid-transition, contact sheets, caption read time, loudness.
    Check {
        #[arg(default_value = "film.yaml")]
        film: PathBuf,
        #[arg(long)]
        no_gate: bool,
    },
}

fn load(film: &Path) -> Result<Project, Value> {
    hivemotion_spec::load(film).map_err(|e| load_error(&e))
}

fn load_error(e: &LoadError) -> Value {
    json!({
        "ok": false,
        "error": e.to_string(),
        "file": e.file(),
        "errors": e.errors(),
    })
}

fn failure(e: anyhow::Error) -> Value {
    json!({ "ok": false, "error": format!("{e:#}") })
}

fn gate(no_gate: bool) -> Gate {
    if no_gate {
        Gate::disabled()
    } else {
        Gate::STANDARD
    }
}

fn run(command: Command) -> Value {
    match command {
        Command::New { dir, name } => match new::create(&dir, &name) {
            Ok(files) => json!({ "ok": true, "dir": dir, "files": files }),
            Err(e) => failure(e),
        },
        Command::Validate { film, write_schema } => {
            let project = match load(&film) {
                Ok(p) => p,
                Err(v) => return v,
            };
            if let Some(dir) = write_schema {
                let write = || -> Result<()> {
                    std::fs::create_dir_all(&dir)?;
                    std::fs::write(
                        dir.join("film.schema.json"),
                        serde_json::to_string_pretty(&hivemotion_spec::film_schema())? + "\n",
                    )?;
                    std::fs::write(
                        dir.join("brand.schema.json"),
                        serde_json::to_string_pretty(&hivemotion_spec::brand_schema())? + "\n",
                    )?;
                    Ok(())
                };
                if let Err(e) = write() {
                    return failure(e);
                }
            }
            let t = &project.timeline;
            json!({
                "ok": true,
                "film": film,
                "title": t.title,
                "length": t.length,
                "fps": t.fps,
                "aspects": t.aspects.values().map(|a| json!({"aspect": a.aspect.name(), "length": a.length()})).collect::<Vec<_>>(),
                "shots": t.shots.len(),
                "captions": t.captions.len(),
                "beats": t.beats,
            })
        }
        Command::Prep { film } => {
            let project = match load(&film) {
                Ok(p) => p,
                Err(v) => return v,
            };
            match hivemotion_render::prep(&project) {
                Ok(r) => json!({ "ok": true, "prep": r }),
                Err(e) => failure(e),
            }
        }
        Command::Render {
            film,
            aspect,
            stills,
        } => {
            let project = match load(&film) {
                Ok(p) => p,
                Err(v) => return v,
            };
            let aspects = match hivemotion_render::aspects(&project, aspect.as_deref()) {
                Ok(a) => a,
                Err(e) => return failure(e),
            };
            let mut reports = Vec::new();
            for a in aspects {
                let result = if stills.is_empty() {
                    hivemotion_render::render_master(&project, a).map(|r| json!(r))
                } else {
                    let plan = &project.timeline.aspects[&a];
                    let times: Result<Vec<f64>> = stills
                        .iter()
                        .map(|s| {
                            let at = TimeRef::parse(s).map_err(|e| anyhow!(e))?;
                            let film_time =
                                hivemotion_spec::time::resolve(&project.timeline.beats, &at)
                                    .map_err(|e| anyhow!(e))?;
                            plan.render_time(film_time)
                                .ok_or_else(|| anyhow!("{s} is not in the {a} cut"))
                        })
                        .collect();
                    times.and_then(|t| {
                        hivemotion_render::render_stills(
                            &project,
                            a,
                            &t,
                            &project.build_paths().stills(),
                        )
                        .map(|r| json!(r))
                    })
                };
                match result {
                    Ok(r) => reports.push(r),
                    Err(e) => return failure(e),
                }
            }
            json!({ "ok": true, "renders": reports })
        }
        Command::Master {
            film,
            aspect,
            no_gate,
        } => {
            let project = match load(&film) {
                Ok(p) => p,
                Err(v) => return v,
            };
            let aspects = match hivemotion_render::aspects(&project, aspect.as_deref()) {
                Ok(a) => a,
                Err(e) => return failure(e),
            };
            let mut reports = Vec::new();
            for a in aspects {
                match hivemotion_audio::master(&project, a, &gate(no_gate)) {
                    Ok(r) => reports.push(r),
                    Err(e) => return failure(e),
                }
            }
            json!({ "ok": true, "masters": reports })
        }
        Command::Deliver { film, preset } => {
            let project = match load(&film) {
                Ok(p) => p,
                Err(v) => return v,
            };
            let presets: Result<Vec<PresetName>> = if preset.is_empty() {
                let has = |a| project.timeline.aspects.contains_key(&a);
                Ok(PresetName::ALL
                    .into_iter()
                    .filter(|p| {
                        hivemotion_deliver::preset(*p)
                            .aspects
                            .iter()
                            .any(|a| has(*a))
                    })
                    .collect())
            } else {
                preset
                    .iter()
                    .map(|p| PresetName::parse(p).ok_or_else(|| anyhow!("no preset named '{p}'")))
                    .collect()
            };
            match presets.and_then(|p| hivemotion_deliver::deliver(&project, &p)) {
                Ok(r) => json!({ "ok": true, "deliver": r }),
                Err(e) => failure(e),
            }
        }
        Command::Check { film, no_gate } => {
            let project = match load(&film) {
                Ok(p) => p,
                Err(v) => return v,
            };
            match hivemotion_check::check(&project, &gate(no_gate)) {
                Ok(r) => json!({ "ok": r.passed, "check": r }),
                Err(e) => failure(e),
            }
        }
    }
}

fn print_text(v: &Value) {
    if v["ok"] == json!(true) {
        println!("ok");
    } else if let Some(e) = v["error"].as_str() {
        eprintln!("error: {e}");
    }
    let mut copy = v.clone();
    if let Some(o) = copy.as_object_mut() {
        o.remove("ok");
        o.remove("error");
    }
    if copy.as_object().is_some_and(|o| !o.is_empty()) {
        println!(
            "{}",
            serde_json::to_string_pretty(&copy).unwrap_or_default()
        );
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = run(cli.command);
    if cli.json {
        println!("{}", serde_json::to_string(&result).unwrap_or_default());
    } else {
        print_text(&result);
    }
    if result["ok"] == json!(true) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
