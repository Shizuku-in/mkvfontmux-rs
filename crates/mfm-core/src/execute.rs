use crate::{
    analyze::{EpisodePlan, FontUse},
    events::{Event, Reporter, Stage},
    settings::{OutputMode, Settings},
};
use ass_fonts::normalize_name;
use mfm_font::{FaceStyle, SubsetRequest};
use mfm_mkv::{Attachment, MuxError, MuxJob, MuxObserver, Muxer, SubtitleTrack};
use rand::{Rng, distr::Alphanumeric};
use rayon::prelude::*;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

/// Result of muxing one episode.
#[derive(Debug, Clone, Serialize)]
pub struct EpisodeOutcome {
    pub output: PathBuf,
    pub attachments: Vec<String>,
    /// Original font name -> name written into the subtitles.
    pub renamed: BTreeMap<String, String>,
    pub warnings: Vec<String>,
    /// Kept temporary directory, when requested.
    pub temp_dir: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExecuteError {
    #[error("no subtitles to add")]
    NoSubtitles,
    #[error("cancelled")]
    Cancelled,
    #[error("{context}: {source}")]
    Io {
        context: String,
        source: std::io::Error,
    },
    #[error(transparent)]
    Mux(MuxError),
}

fn io_err(context: impl Into<String>) -> impl FnOnce(std::io::Error) -> ExecuteError {
    let context = context.into();
    move |source| ExecuteError::Io { context, source }
}

/// Subset fonts, rewrite subtitles, and mux episode number `episode` of a batch.
pub fn execute(
    episode: usize,
    plan: &EpisodePlan,
    settings: &Settings,
    muxer: &dyn Muxer,
    reporter: &dyn Reporter,
) -> Result<EpisodeOutcome, ExecuteError> {
    if plan.subtitles.is_empty() {
        return Err(ExecuteError::NoSubtitles);
    }
    let cancelled = || {
        if reporter.is_cancelled() {
            Err(ExecuteError::Cancelled)
        } else {
            Ok(())
        }
    };
    let temp = tempfile::Builder::new()
        .prefix("mkvfontmux-")
        .tempdir()
        .map_err(io_err("cannot create temporary directory"))?;
    let mut warnings = Vec::new();

    reporter.report(Event::Stage {
        episode,
        stage: Stage::Subsetting,
    });
    let families: BTreeMap<String, String> = if settings.subset && settings.rename {
        let mut taken = BTreeSet::new();
        plan.fonts
            .iter()
            .map(|f| f.key.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|key| (key, random_family(&mut taken)))
            .collect()
    } else {
        BTreeMap::new()
    };
    let (attachments, failed) = if settings.subset {
        subset_fonts(episode, plan, &families, temp.path(), reporter)?
    } else {
        (original_fonts(plan), Vec::new())
    };
    warnings.extend(failed);
    cancelled()?;

    reporter.report(Event::Stage {
        episode,
        stage: Stage::Rewriting,
    });
    let mut subtitles = Vec::new();
    for (i, subtitle) in plan.subtitles.iter().enumerate() {
        let file_name = subtitle.path.file_name().unwrap_or_default();
        let text = if families.is_empty() {
            subtitle.text.clone()
        } else {
            let rewritten = ass_fonts::rewrite_font_names(&subtitle.text, |site| {
                families.get(&normalize_name(site.name)).cloned()
            });
            for d in rewritten.diagnostics {
                warnings.push(format!(
                    "{}:{}: {}",
                    file_name.to_string_lossy(),
                    d.line,
                    d.message
                ));
            }
            rewritten.text
        };
        let path = temp.path().join(file_name);
        let mut bytes = Vec::with_capacity(text.len() + 3);
        if !text.starts_with('\u{feff}') {
            bytes.extend_from_slice("\u{feff}".as_bytes());
        }
        bytes.extend_from_slice(text.as_bytes());
        std::fs::write(&path, bytes).map_err(io_err(format!("cannot write {}", path.display())))?;
        subtitles.push(SubtitleTrack {
            path,
            language: subtitle.language.clone(),
            name: None,
            default: i == 0,
        });
    }
    cancelled()?;

    reporter.report(Event::Stage {
        episode,
        stage: Stage::Muxing,
    });
    let (output, staging) = output_paths(&plan.mkv, &settings.output)?;
    let job = MuxJob {
        input: plan.mkv.clone(),
        output: staging.clone().unwrap_or_else(|| output.clone()),
        subtitles,
        attachments: attachments.clone(),
    };
    let mut observer = Observer { episode, reporter };
    let outcome = muxer.mux(&job, &mut observer).map_err(|e| match e {
        MuxError::Cancelled => ExecuteError::Cancelled,
        e => ExecuteError::Mux(e),
    })?;
    warnings.extend(outcome.warnings);

    reporter.report(Event::Stage {
        episode,
        stage: Stage::Finalizing,
    });
    if let Some(staging) = staging {
        std::fs::rename(&staging, &output)
            .map_err(io_err(format!("cannot replace {}", output.display())))?;
    }
    let temp_dir = if settings.keep_temp {
        Some(temp.keep())
    } else {
        None
    };

    Ok(EpisodeOutcome {
        output,
        attachments: attachments.into_iter().map(|a| a.name).collect(),
        renamed: plan
            .fonts
            .iter()
            .filter_map(|f| families.get(&f.key).map(|r| (f.name.clone(), r.clone())))
            .collect(),
        warnings,
        temp_dir,
    })
}

/// Final output path, and a staging path in the same directory when the
/// source is replaced (so the replacement is a rename on the same volume).
fn output_paths(mkv: &Path, mode: &OutputMode) -> Result<(PathBuf, Option<PathBuf>), ExecuteError> {
    let dir = mkv.parent().unwrap_or(Path::new("."));
    let name = mkv.file_name().unwrap_or_default();
    match mode {
        OutputMode::Subdirectory { name: sub } => {
            let out_dir = dir.join(sub);
            std::fs::create_dir_all(&out_dir)
                .map_err(io_err(format!("cannot create {}", out_dir.display())))?;
            Ok((out_dir.join(name), None))
        }
        OutputMode::Overwrite => {
            let stem = mkv.file_stem().unwrap_or_default().to_string_lossy();
            Ok((
                mkv.to_owned(),
                Some(dir.join(format!(".{stem}.mkvfontmux-tmp.mkv"))),
            ))
        }
    }
}

fn random_family(taken: &mut BTreeSet<String>) -> String {
    loop {
        let mut rng = rand::rng();
        // Start with a letter: some renderers dislike names beginning with a digit.
        let first = rng.random_range(b'A'..=b'Z') as char;
        let name: String = std::iter::once(first)
            .chain(rng.sample_iter(&Alphanumeric).take(9).map(char::from))
            .collect();
        if taken.insert(name.clone()) {
            return name;
        }
    }
}

struct SubsetTask<'a> {
    label: String,
    request: SubsetRequest,
    stem: String,
    uses: Vec<&'a FontUse>,
}

fn subset_fonts(
    episode: usize,
    plan: &EpisodePlan,
    families: &BTreeMap<String, String>,
    dir: &Path,
    reporter: &dyn Reporter,
) -> Result<(Vec<Attachment>, Vec<String>), ExecuteError> {
    // With renaming, each (name, face) is its own font; without, faces are shared.
    let mut tasks: BTreeMap<(Option<String>, PathBuf, u32), SubsetTask> = BTreeMap::new();
    for font in &plan.fonts {
        let family = families.get(&font.key).cloned();
        let task = tasks
            .entry((family.clone(), font.face.path.clone(), font.face.face_index))
            .or_insert_with(|| {
                let mut stem = font
                    .face
                    .path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                if font.face.face_index > 0 {
                    stem.push_str(&format!("_{}", font.face.face_index));
                }
                if let Some(family) = &family {
                    stem.push('_');
                    stem.push_str(family);
                }
                SubsetTask {
                    label: font.name.clone(),
                    request: SubsetRequest {
                        path: font.face.path.clone(),
                        face_index: font.face.face_index,
                        characters: BTreeSet::new(),
                        family,
                        style: FaceStyle {
                            weight: font.face.weight,
                            italic: font.face.italic,
                        },
                    },
                    stem,
                    uses: Vec::new(),
                }
            });
        task.request.characters.extend(&font.characters);
        task.uses.push(font);
    }

    let results: Vec<_> = tasks
        .into_values()
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|task| {
            if reporter.is_cancelled() {
                return Err(ExecuteError::Cancelled);
            }
            let result = mfm_font::subset_face(&task.request).and_then(|subset| {
                let name = format!("{}.{}", task.stem, subset.extension);
                let path = dir.join(&name);
                std::fs::write(&path, &subset.data).map_err(|source| {
                    mfm_font::SubsetError::Io {
                        path: path.clone(),
                        source,
                    }
                })?;
                reporter.report(Event::FontReady {
                    episode,
                    font: task.label.clone(),
                    attachment: name.clone(),
                    bytes: subset.data.len() as u64,
                });
                Ok(Attachment {
                    path,
                    name,
                    mime_type: subset.mime_type.into(),
                })
            });
            Ok(result.map_err(|e| format!("font \"{}\" not embedded: {e}", task.label)))
        })
        .collect::<Result<_, _>>()?;

    let mut attachments = Vec::new();
    let mut failed = Vec::new();
    for result in results {
        match result {
            Ok(attachment) => attachments.push(attachment),
            Err(message) => {
                reporter.report(Event::Warning {
                    episode: Some(episode),
                    message: message.clone(),
                });
                failed.push(message);
            }
        }
    }
    Ok((attachments, failed))
}

/// Attach the original font files unchanged (subsetting disabled).
fn original_fonts(plan: &EpisodePlan) -> Vec<Attachment> {
    let paths: BTreeSet<&Path> = plan.fonts.iter().map(|f| f.face.path.as_path()).collect();
    paths
        .into_iter()
        .map(|path| {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let otf = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("otf") || e.eq_ignore_ascii_case("otc"));
            Attachment {
                path: path.to_owned(),
                name,
                mime_type: if otf {
                    "application/vnd.ms-opentype"
                } else {
                    "application/x-truetype-font"
                }
                .into(),
            }
        })
        .collect()
}

struct Observer<'a> {
    episode: usize,
    reporter: &'a dyn Reporter,
}

impl MuxObserver for Observer<'_> {
    fn progress(&mut self, percent: u8) {
        self.reporter.report(Event::MuxProgress {
            episode: self.episode,
            percent,
        });
    }

    fn cancelled(&self) -> bool {
        self.reporter.is_cancelled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_families_are_unique_and_start_with_a_letter() {
        let mut taken = BTreeSet::new();
        for _ in 0..100 {
            let name = random_family(&mut taken);
            assert_eq!(name.len(), 10);
            assert!(name.starts_with(|c: char| c.is_ascii_uppercase()));
        }
        assert_eq!(taken.len(), 100);
    }

    #[test]
    fn overwrite_stages_next_to_source() {
        let (out, staging) =
            output_paths(Path::new("/v/Ep 1.mkv"), &OutputMode::Overwrite).unwrap();
        assert_eq!(out, Path::new("/v/Ep 1.mkv"));
        assert_eq!(staging.unwrap(), Path::new("/v/.Ep 1.mkvfontmux-tmp.mkv"));
    }
}
