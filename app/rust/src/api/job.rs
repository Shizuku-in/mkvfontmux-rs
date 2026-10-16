//! Discovering episodes, preflight analysis, and running a batch.

use super::{library::FontLibrary, settings::SettingsDto};
use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use mfm_core::{
    Episode, EpisodePlan, Event, FaceRef, FontOverrides, MissingKind, MkvmergeMuxer, Muxer,
    MuxerKind, NativeMuxer, Reporter, Settings, Stage, SubtitleInput, normalize_name,
};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Debug, Clone)]
pub struct EpisodeDto {
    pub mkv: String,
    pub subtitles: Vec<SubtitleDto>,
}

#[derive(Debug, Clone)]
pub struct SubtitleDto {
    pub path: String,
    /// BCP 47 tag; editable before running.
    pub language: String,
}

impl From<Episode> for EpisodeDto {
    fn from(e: Episode) -> Self {
        Self {
            mkv: e.mkv.display().to_string(),
            subtitles: e
                .subtitles
                .into_iter()
                .map(|s| SubtitleDto {
                    path: s.path.display().to_string(),
                    language: s.language,
                })
                .collect(),
        }
    }
}

impl From<EpisodeDto> for Episode {
    fn from(e: EpisodeDto) -> Self {
        Self {
            mkv: e.mkv.into(),
            subtitles: e
                .subtitles
                .into_iter()
                .map(|s| SubtitleInput {
                    path: s.path.into(),
                    language: s.language,
                })
                .collect(),
        }
    }
}

/// A user's choice of face for a font name, applied to the whole batch.
#[derive(Debug, Clone)]
pub struct FontOverride {
    pub name: String,
    pub path: String,
    pub face_index: u32,
}

fn overrides(list: Vec<FontOverride>) -> FontOverrides {
    list.into_iter()
        .map(|o| {
            (
                normalize_name(&o.name),
                FaceRef {
                    path: o.path.into(),
                    face_index: o.face_index,
                },
            )
        })
        .collect()
}

/// Pair `.mkv` files (or every `.mkv` in directories) with their subtitles.
pub fn discover(paths: Vec<String>, settings: SettingsDto) -> anyhow::Result<Vec<EpisodeDto>> {
    let settings = Settings::from(settings);
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    Ok(mfm_core::discover(&paths, &settings.languages)?
        .into_iter()
        .map(EpisodeDto::from)
        .collect())
}

/// Language tag the settings assign to a subtitle file name.
#[frb(sync)]
pub fn detect_language(file_name: String, settings: SettingsDto) -> String {
    Settings::from(settings)
        .languages
        .detect(&file_name)
        .to_owned()
}

// ---- preflight ----

#[derive(Debug, Clone)]
pub struct PlanSummary {
    pub subtitles: Vec<SubtitleSummary>,
    pub fonts: Vec<FontSummary>,
    pub missing: Vec<MissingSummary>,
    pub glyph_gaps: Vec<GlyphGapSummary>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SubtitleSummary {
    pub path: String,
    pub language: String,
    pub encoding: String,
}

#[derive(Debug, Clone)]
pub struct FontSummary {
    pub name: String,
    pub path: String,
    pub face_index: u32,
    pub characters: u32,
    pub ambiguous: bool,
    pub fallback_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MissingSummary {
    pub name: String,
    pub weight: u16,
    pub italic: bool,
    /// The family exists but not in the requested weight/slant.
    pub style_only: bool,
    /// `file:line,line` entries.
    pub uses: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GlyphGapSummary {
    pub font: String,
    pub path: String,
    pub characters: String,
}

impl From<&EpisodePlan> for PlanSummary {
    fn from(p: &EpisodePlan) -> Self {
        let name = |path: &std::path::Path| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        };
        Self {
            subtitles: p
                .subtitles
                .iter()
                .map(|s| SubtitleSummary {
                    path: s.path.display().to_string(),
                    language: s.language.clone(),
                    encoding: s.encoding.clone(),
                })
                .collect(),
            fonts: p
                .fonts
                .iter()
                .map(|f| FontSummary {
                    name: f.name.clone(),
                    path: f.face.path.display().to_string(),
                    face_index: f.face.face_index,
                    characters: f.characters.len() as u32,
                    ambiguous: f.ambiguous,
                    fallback_name: f.fallback_name.clone(),
                })
                .collect(),
            missing: p
                .missing
                .iter()
                .map(|m| MissingSummary {
                    name: m.name.clone(),
                    weight: m.weight,
                    italic: m.italic,
                    style_only: m.reason == MissingKind::StyleNotFound,
                    uses: m
                        .uses
                        .iter()
                        .map(|(path, lines)| {
                            let lines: Vec<String> = lines.iter().map(usize::to_string).collect();
                            format!("{}:{}", name(path), lines.join(","))
                        })
                        .collect(),
                })
                .collect(),
            glyph_gaps: p
                .missing_glyphs
                .iter()
                .map(|g| GlyphGapSummary {
                    font: g.font.clone(),
                    path: g.face.display().to_string(),
                    characters: g.characters.iter().collect(),
                })
                .collect(),
            warnings: p.warnings.clone(),
        }
    }
}

/// Preflight one episode without writing anything.
pub fn analyze(
    library: &FontLibrary,
    episode: EpisodeDto,
    settings: SettingsDto,
    overrides_list: Vec<FontOverride>,
) -> PlanSummary {
    let plan = mfm_core::analyze(
        &episode.into(),
        &library.index,
        &settings.into(),
        &overrides(overrides_list),
    );
    PlanSummary::from(&plan)
}

// ---- running ----

#[frb(opaque)]
#[derive(Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    #[frb(sync)]
    pub fn new() -> CancelToken {
        CancelToken::default()
    }

    #[frb(sync)]
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStage {
    Analyzing,
    Subsetting,
    Rewriting,
    Muxing,
    Finalizing,
}

#[derive(Debug, Clone)]
pub enum RunEvent {
    Stage {
        episode: u32,
        stage: RunStage,
    },
    FontReady {
        episode: u32,
        font: String,
        attachment: String,
        bytes: u64,
    },
    Progress {
        episode: u32,
        percent: u8,
    },
    Warning {
        episode: Option<u32>,
        message: String,
    },
    Done {
        episode: u32,
        output: String,
        warnings: Vec<String>,
    },
    Failed {
        episode: u32,
        error: String,
    },
    Skipped {
        episode: u32,
        reason: String,
    },
    Finished {
        succeeded: u32,
        failed: u32,
        cancelled: bool,
    },
}

struct Sink {
    sink: StreamSink<RunEvent>,
    cancel: Arc<AtomicBool>,
}

impl Sink {
    fn send(&self, event: RunEvent) {
        // The UI may have gone away; nothing useful to do on failure.
        let _ = self.sink.add(event);
    }
}

impl Reporter for Sink {
    fn report(&self, event: Event) {
        let mapped = match event {
            Event::IndexProgress { .. } => return,
            Event::Stage { episode, stage } => RunEvent::Stage {
                episode: episode as u32,
                stage: match stage {
                    Stage::Subsetting => RunStage::Subsetting,
                    Stage::Rewriting => RunStage::Rewriting,
                    Stage::Muxing => RunStage::Muxing,
                    Stage::Finalizing => RunStage::Finalizing,
                },
            },
            Event::FontReady {
                episode,
                font,
                attachment,
                bytes,
            } => RunEvent::FontReady {
                episode: episode as u32,
                font,
                attachment,
                bytes,
            },
            Event::MuxProgress { episode, percent } => RunEvent::Progress {
                episode: episode as u32,
                percent,
            },
            Event::Warning { episode, message } => RunEvent::Warning {
                episode: episode.map(|e| e as u32),
                message,
            },
        };
        self.send(mapped);
    }

    fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
}

/// Process `episodes` in order, streaming [`RunEvent`]s. Episode numbers in
/// events are indices into `episodes`. Ends with [`RunEvent::Finished`].
pub fn run_batch(
    library: &FontLibrary,
    episodes: Vec<EpisodeDto>,
    settings: SettingsDto,
    overrides_list: Vec<FontOverride>,
    cancel: &CancelToken,
    sink: StreamSink<RunEvent>,
) {
    let settings = Settings::from(settings);
    let overrides = overrides(overrides_list);
    let sink = Sink {
        sink,
        cancel: cancel.0.clone(),
    };
    let muxer: Box<dyn Muxer> = match settings.muxer {
        MuxerKind::Native => Box::new(NativeMuxer),
        MuxerKind::Mkvmerge => match MkvmergeMuxer::locate(settings.mkvmerge.as_deref()) {
            Some(m) => Box::new(m),
            None => {
                for i in 0..episodes.len() {
                    sink.send(RunEvent::Failed {
                        episode: i as u32,
                        error: "mkvmerge not found".into(),
                    });
                }
                sink.send(RunEvent::Finished {
                    succeeded: 0,
                    failed: episodes.len() as u32,
                    cancelled: false,
                });
                return;
            }
        },
    };

    let (mut succeeded, mut failed) = (0, 0);
    for (i, episode) in episodes.into_iter().enumerate() {
        let n = i as u32;
        if sink.is_cancelled() {
            break;
        }
        if episode.subtitles.is_empty() {
            sink.send(RunEvent::Skipped {
                episode: n,
                reason: "no subtitles".into(),
            });
            continue;
        }
        sink.send(RunEvent::Stage {
            episode: n,
            stage: RunStage::Analyzing,
        });
        let plan = mfm_core::analyze(&episode.into(), &library.index, &settings, &overrides);
        match mfm_core::execute(i, &plan, &settings, muxer.as_ref(), &sink) {
            Ok(outcome) => {
                succeeded += 1;
                let mut warnings = plan.warnings.clone();
                warnings.extend(outcome.warnings);
                sink.send(RunEvent::Done {
                    episode: n,
                    output: outcome.output.display().to_string(),
                    warnings,
                });
            }
            Err(mfm_core::ExecuteError::Cancelled) => break,
            Err(e) => {
                failed += 1;
                sink.send(RunEvent::Failed {
                    episode: n,
                    error: e.to_string(),
                });
            }
        }
    }
    sink.send(RunEvent::Finished {
        succeeded,
        failed,
        cancelled: sink.is_cancelled(),
    });
}
