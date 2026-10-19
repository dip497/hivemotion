use std::path::{Path, PathBuf};

use crate::film::Aspect;

/// Where every stage writes, under the film's `build/` folder.
#[derive(Debug, Clone)]
pub struct BuildPaths {
    pub root: PathBuf,
}

impl BuildPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        BuildPaths { root: root.into() }
    }

    /// Clips cut to their shots and the brand's fonts, read by the renderer.
    pub fn media(&self) -> PathBuf {
        self.root.join("media")
    }

    /// The picture as rendered: ProRes 4444, no sound.
    pub fn render(&self, aspect: Aspect) -> PathBuf {
        self.root.join(format!("render-{}.mov", aspect.file_name()))
    }

    /// The music laid out on this aspect's timeline, before mastering.
    pub fn music_mix(&self, aspect: Aspect) -> PathBuf {
        self.root
            .join(format!("music-mix-{}.wav", aspect.file_name()))
    }

    /// The mastered sound.
    pub fn audio(&self, aspect: Aspect) -> PathBuf {
        self.root.join(format!("audio-{}.wav", aspect.file_name()))
    }

    /// The master: the render with the mastered sound.
    pub fn master(&self, aspect: Aspect) -> PathBuf {
        self.root.join(format!("master-{}.mov", aspect.file_name()))
    }

    pub fn deliver(&self) -> PathBuf {
        self.root.join("deliver")
    }

    pub fn check(&self) -> PathBuf {
        self.root.join("check")
    }

    pub fn stills(&self) -> PathBuf {
        self.root.join("stills")
    }
}

impl AsRef<Path> for BuildPaths {
    fn as_ref(&self) -> &Path {
        &self.root
    }
}
