//! Adding subtitle tracks and font attachments to Matroska files.
//!
//! [`Muxer`] abstracts the backend. [`MkvmergeMuxer`] drives an external
//! `mkvmerge`; a native backend is planned alongside it.

mod mkvmerge;

pub use mkvmerge::MkvmergeMuxer;

use std::path::PathBuf;

/// One remux: copy `input`, append subtitle tracks and attachments, write `output`.
#[derive(Debug, Clone)]
pub struct MuxJob {
    pub input: PathBuf,
    pub output: PathBuf,
    pub subtitles: Vec<SubtitleTrack>,
    pub attachments: Vec<Attachment>,
}

/// An external ASS/SSA file to add as a subtitle track.
#[derive(Debug, Clone)]
pub struct SubtitleTrack {
    pub path: PathBuf,
    /// BCP 47 language tag, such as `zh-Hans` or `ja`.
    pub language: String,
    pub name: Option<String>,
    pub default: bool,
}

/// A file to attach, typically a font.
#[derive(Debug, Clone)]
pub struct Attachment {
    pub path: PathBuf,
    /// Name stored in the Matroska file.
    pub name: String,
    pub mime_type: String,
}

/// Result of a successful remux.
#[derive(Debug, Default, Clone)]
pub struct MuxOutcome {
    /// Non-fatal messages reported by the backend.
    pub warnings: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum MuxError {
    #[error("failed to start {program}: {source}")]
    Spawn {
        program: PathBuf,
        source: std::io::Error,
    },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("muxing was cancelled")]
    Cancelled,
    #[error("{backend} failed: {message}")]
    Failed {
        backend: &'static str,
        message: String,
    },
}

/// Callbacks a backend uses while muxing.
pub trait MuxObserver {
    /// Progress in percent, 0–100.
    fn progress(&mut self, _percent: u8) {}
    /// Polled regularly; returning `true` aborts the job and removes partial output.
    fn cancelled(&self) -> bool {
        false
    }
}

/// An observer that ignores progress and never cancels.
pub struct NoObserver;
impl MuxObserver for NoObserver {}

/// A remuxing backend.
pub trait Muxer: Send + Sync {
    /// Short identifier for logs and settings.
    fn name(&self) -> &'static str;
    fn mux(&self, job: &MuxJob, observer: &mut dyn MuxObserver) -> Result<MuxOutcome, MuxError>;
}
