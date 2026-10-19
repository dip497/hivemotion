//! film.yaml and brand.yaml: the types, their JSON Schema, and validation whose errors name the
//! field (`shots[3].from: no beat named 'drp'`).

pub mod brand;
pub mod film;
pub mod paths;
pub mod time;
pub mod timeline;

use std::fmt;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

pub use brand::Brand;
pub use film::{Aspect, Film};
pub use paths::BuildPaths;
pub use time::TimeRef;
pub use timeline::{AspectPlan, Timeline};

/// One problem in a spec file, at a field path such as `shots[3].from`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpecError {
    pub path: String,
    pub message: String,
}

impl fmt::Display for SpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("cannot read {file}: {source}")]
    Read {
        file: PathBuf,
        source: std::io::Error,
    },
    #[error("{file}: {}", errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n  "))]
    Invalid {
        file: PathBuf,
        errors: Vec<SpecError>,
    },
}

impl LoadError {
    pub fn errors(&self) -> &[SpecError] {
        match self {
            LoadError::Read { .. } => &[],
            LoadError::Invalid { errors, .. } => errors,
        }
    }

    pub fn file(&self) -> &Path {
        match self {
            LoadError::Read { file, .. } | LoadError::Invalid { file, .. } => file,
        }
    }
}

/// A loaded, checked film with its brand.
#[derive(Debug, Clone)]
pub struct Project {
    pub film_file: PathBuf,
    pub dir: PathBuf,
    pub film: Film,
    pub brand: Brand,
    pub brand_dir: PathBuf,
    pub timeline: Timeline,
}

impl Project {
    pub fn font_files(&self) -> Vec<PathBuf> {
        let f = &self.brand.fonts;
        f.sans
            .files
            .iter()
            .chain(&f.mono.files)
            .map(|p| self.brand_dir.join(p))
            .collect()
    }

    pub fn build_paths(&self) -> BuildPaths {
        BuildPaths::new(self.dir.join("build"))
    }

    pub fn mark_file(&self) -> Option<PathBuf> {
        self.brand.mark.as_ref().map(|m| self.brand_dir.join(m))
    }
}

fn parse<T: DeserializeOwned>(file: &Path) -> Result<T, LoadError> {
    let text = std::fs::read_to_string(file).map_err(|source| LoadError::Read {
        file: file.to_path_buf(),
        source,
    })?;
    parse_text(&text).map_err(|errors| LoadError::Invalid {
        file: file.to_path_buf(),
        errors,
    })
}

fn parse_text<T: DeserializeOwned>(text: &str) -> Result<T, Vec<SpecError>> {
    let de = serde_yaml::Deserializer::from_str(text);
    serde_path_to_error::deserialize(de).map_err(|err| {
        let path = err.path().to_string();
        let inner = err.into_inner().to_string();
        vec![SpecError {
            path: if path == "." { "(top)".into() } else { path },
            message: inner,
        }]
    })
}

/// Read film.yaml and the brand.yaml it names, and check both.
pub fn load(film_file: &Path) -> Result<Project, LoadError> {
    let film: Film = parse(film_file)?;
    let dir = film_file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let brand_file = dir.join(&film.brand);
    let brand: Brand = parse(&brand_file)?;
    let brand_dir = brand_file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();

    let mut brand_errors = timeline::check_brand(&brand);
    for (name, font) in [("sans", &brand.fonts.sans), ("mono", &brand.fonts.mono)] {
        for (i, f) in font.files.iter().enumerate() {
            if !brand_dir.join(f).is_file() {
                brand_errors.push(SpecError {
                    path: format!("fonts.{name}.files[{i}]"),
                    message: format!("no file at {f}"),
                });
            }
        }
    }
    if let Some(mark) = &brand.mark {
        if !brand_dir.join(mark).is_file() {
            brand_errors.push(SpecError {
                path: "mark".into(),
                message: format!("no file at {mark}"),
            });
        }
    }
    if !brand_errors.is_empty() {
        return Err(LoadError::Invalid {
            file: brand_file,
            errors: brand_errors,
        });
    }

    let timeline = timeline::build(&film, &dir).map_err(|errors| LoadError::Invalid {
        file: film_file.to_path_buf(),
        errors,
    })?;
    Ok(Project {
        film_file: film_file.to_path_buf(),
        dir,
        film,
        brand,
        brand_dir,
        timeline,
    })
}

pub fn film_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(Film)).expect("schema serializes")
}

pub fn brand_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(Brand)).expect("schema serializes")
}
