use serde::Serialize;

/// Pipeline stage of one episode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Subsetting,
    Rewriting,
    Muxing,
    Finalizing,
}

/// Progress notifications for frontends.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    IndexProgress {
        scanned: usize,
        to_scan: usize,
        cached: usize,
    },
    Stage {
        episode: usize,
        stage: Stage,
    },
    FontReady {
        episode: usize,
        font: String,
        attachment: String,
        bytes: u64,
    },
    MuxProgress {
        episode: usize,
        percent: u8,
    },
    Warning {
        episode: Option<usize>,
        message: String,
    },
}

/// Receives events and answers cancellation polls. Must be shareable across
/// the threads used for parallel subsetting.
pub trait Reporter: Sync {
    fn report(&self, event: Event);
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// A reporter that discards events and never cancels.
pub struct Silent;
impl Reporter for Silent {
    fn report(&self, _event: Event) {}
}
