use crate::{discover::Episode, settings::Settings};
use ass_fonts::{
    FontFace, FontIndex, FontReference, MissingReason, Resolution, ResolveOptions, ResolveReport,
    SynthesisPolicy, WeightMatching, normalize_name,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

/// Encoding suffixes stripped by smart matching, as in the original tool.
const SMART_SUFFIXES: [&str; 5] = ["_gbk", "_gb2312", "_big5", "_jis", "_kr"];

/// A specific face in a font file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FaceRef {
    pub path: PathBuf,
    pub face_index: u32,
}

/// Faces chosen by the user, keyed by font name (normalized with
/// [`ass_fonts::normalize_name`]). They replace automatic resolution for every
/// weight and slant of that name, and may point at files outside the index.
pub type FontOverrides = BTreeMap<String, FaceRef>;

/// Everything needed to mux one episode, computed without writing files.
#[derive(Debug, Clone, Serialize)]
pub struct EpisodePlan {
    pub mkv: PathBuf,
    pub subtitles: Vec<PreparedSubtitle>,
    /// One entry per (subtitle font name, selected face).
    pub fonts: Vec<FontUse>,
    pub missing: Vec<MissingFont>,
    /// Characters that selected faces do not map; they will render with fallback fonts.
    pub missing_glyphs: Vec<GlyphGap>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreparedSubtitle {
    pub path: PathBuf,
    pub language: String,
    /// Decoded text; rewritten and written as UTF-8 when muxing.
    #[serde(skip)]
    pub text: String,
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FontUse {
    /// Font name as first written in the subtitles.
    pub name: String,
    /// Normalized name used for rewriting.
    pub key: String,
    pub face: FontFace,
    pub characters: BTreeSet<char>,
    /// The face was chosen among several equally good candidates.
    pub ambiguous: bool,
    /// The name only matched after stripping an encoding suffix.
    pub fallback_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MissingFont {
    pub name: String,
    pub weight: u16,
    pub italic: bool,
    pub reason: MissingKind,
    /// Subtitle files and line numbers that use the font.
    pub uses: Vec<(PathBuf, Vec<usize>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingKind {
    NameNotFound,
    StyleNotFound,
}

#[derive(Debug, Clone, Serialize)]
pub struct GlyphGap {
    pub font: String,
    pub face: PathBuf,
    pub characters: Vec<char>,
}

impl EpisodePlan {
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty() && self.missing_glyphs.is_empty()
    }
}

/// Resolve the fonts of an episode's subtitles against `index`, applying `overrides`.
pub fn analyze(
    episode: &Episode,
    index: &FontIndex,
    settings: &Settings,
    overrides: &FontOverrides,
) -> EpisodePlan {
    let mut plan = EpisodePlan {
        mkv: episode.mkv.clone(),
        subtitles: Vec::new(),
        fonts: Vec::new(),
        missing: Vec::new(),
        missing_glyphs: Vec::new(),
        warnings: Vec::new(),
    };
    let options = ResolveOptions {
        weight_matching: if settings.nearest_weight {
            WeightMatching::Nearest
        } else {
            WeightMatching::Exact
        },
        synthesis: SynthesisPolicy {
            bold: settings.allow_synthesis,
            italic: settings.allow_synthesis,
        },
    };
    let ignored: BTreeSet<String> = settings
        .ignored_fonts
        .iter()
        .map(|n| normalize_name(n))
        .collect();

    let mut chosen: BTreeMap<&str, FontFace> = BTreeMap::new();
    for (name, face) in overrides {
        match load_face(index, face) {
            Ok(found) => {
                chosen.insert(name.as_str(), found);
            }
            Err(e) => plan.warnings.push(format!(
                "override for \"{name}\" ignored: {}#{}: {e}",
                face.path.display(),
                face.face_index
            )),
        }
    }

    // (key, path, face index) -> use
    let mut uses: BTreeMap<(String, PathBuf, u32), FontUse> = BTreeMap::new();
    let mut missing: BTreeMap<(String, u16, bool), MissingFont> = BTreeMap::new();
    let mut resolved_reports = Vec::new();

    for input in &episode.subtitles {
        let file_name = input
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let bytes = match std::fs::read(&input.path) {
            Ok(bytes) => bytes,
            Err(e) => {
                plan.warnings.push(format!("{file_name}: cannot read: {e}"));
                continue;
            }
        };
        let decoded = match ass_fonts::decode_subtitle(&bytes) {
            Ok(decoded) => decoded,
            Err(e) => {
                plan.warnings
                    .push(format!("{file_name}: cannot decode: {e}"));
                continue;
            }
        };
        let extracted = ass_fonts::extract_fonts(&decoded.text);
        for d in &extracted.diagnostics {
            plan.warnings
                .push(format!("{file_name}:{}: {}", d.line, d.message));
        }
        let (manual, references): (Vec<FontReference>, Vec<FontReference>) = extracted
            .references
            .into_iter()
            .filter(|r| !ignored.contains(&normalize_name(&r.name)))
            .partition(|r| chosen.contains_key(normalize_name(&r.name).as_str()));
        let mut report = index.resolve_with_options(&references, options);

        if settings.smart_match {
            retry_without_suffix(index, options, &mut report);
        }
        for reference in manual {
            let face = chosen[normalize_name(&reference.name).as_str()].clone();
            report.resolved.push(Resolution {
                reference,
                candidates: vec![face],
                missing_reason: None,
                available_variants: Vec::new(),
                matches: Vec::new(),
            });
        }

        for resolution in report.resolved.iter().chain(&report.ambiguous) {
            let reference = &resolution.reference;
            let Some(face) = resolution.candidates.first() else {
                continue;
            };
            // Identical copies (the same font installed twice) are not a real choice.
            let others: Vec<&FontFace> = resolution.candidates[1..]
                .iter()
                .filter(|f| !same_font(f, face))
                .collect();
            let ambiguous = !others.is_empty();
            if ambiguous {
                let others: Vec<String> = others
                    .iter()
                    .map(|f| format!("{}#{}", f.path.display(), f.face_index))
                    .collect();
                let message = format!(
                    "\"{}\" matches several different faces; using {}#{} (also: {})",
                    reference.name.trim_start_matches('@'),
                    face.path.display(),
                    face.face_index,
                    others.join(", ")
                );
                if !plan.warnings.contains(&message) {
                    plan.warnings.push(message);
                }
            }
            let key = normalize_name(&reference.name);
            let entry = uses
                .entry((key.clone(), face.path.clone(), face.face_index))
                .or_insert_with(|| FontUse {
                    name: reference.name.trim_start_matches('@').to_owned(),
                    key,
                    face: face.clone(),
                    characters: BTreeSet::new(),
                    ambiguous,
                    fallback_name: None,
                });
            entry.ambiguous |= ambiguous;
            entry.characters.extend(
                reference
                    .characters
                    .iter()
                    .chain(&reference.whitespace)
                    .map(|u| u.character),
            );
        }
        for resolution in &report.missing {
            let r = &resolution.reference;
            let entry = missing
                .entry((normalize_name(&r.name), r.weight, r.italic))
                .or_insert_with(|| MissingFont {
                    name: r.name.trim_start_matches('@').to_owned(),
                    weight: r.weight,
                    italic: r.italic,
                    reason: match resolution.missing_reason {
                        Some(MissingReason::StyleNotFound) => MissingKind::StyleNotFound,
                        _ => MissingKind::NameNotFound,
                    },
                    uses: Vec::new(),
                });
            entry.uses.push((input.path.clone(), r.lines.clone()));
        }
        // Only the chosen candidate is embedded, so only it matters for coverage.
        for resolution in &mut report.ambiguous {
            resolution.candidates.truncate(1);
            resolution.matches.truncate(1);
        }
        resolved_reports.push(report);
        plan.subtitles.push(PreparedSubtitle {
            path: input.path.clone(),
            language: input.language.clone(),
            text: decoded.text,
            encoding: decoded.encoding.name().to_owned(),
        });
    }

    // Record which uses came from smart matching (their reference name kept the suffix).
    for use_ in uses.values_mut() {
        if let Some(base) = strip_suffix(&use_.key)
            && !use_
                .face
                .names
                .iter()
                .any(|n| normalize_name(n) == use_.key)
        {
            use_.fallback_name = Some(base.to_owned());
        }
    }

    for report in &resolved_reports {
        for check in report.check_coverage().incomplete {
            plan.missing_glyphs.push(GlyphGap {
                font: check.reference.name.clone(),
                face: check.candidate.path.clone(),
                characters: check
                    .missing_characters
                    .iter()
                    .map(|u| u.character)
                    .collect(),
            });
        }
    }
    plan.missing_glyphs
        .sort_by(|a, b| (&a.font, &a.face).cmp(&(&b.font, &b.face)));
    plan.missing_glyphs.dedup_by(|a, b| {
        if a.font == b.font && a.face == b.face {
            b.characters.append(&mut a.characters);
            b.characters.sort_unstable();
            b.characters.dedup();
            true
        } else {
            false
        }
    });

    plan.fonts = uses.into_values().collect();
    plan.missing = missing.into_values().collect();
    plan
}

/// Find an override's face in the index, or scan its file when it is outside the index.
fn load_face(index: &FontIndex, face: &FaceRef) -> Result<FontFace, String> {
    if let Some(found) = index
        .faces()
        .iter()
        .find(|f| f.path == face.path && f.face_index == face.face_index)
    {
        return Ok(found.clone());
    }
    let (faces, issues) = ass_fonts::scan_font_file(&face.path);
    faces
        .into_iter()
        .find(|f| f.face_index == face.face_index)
        .ok_or_else(|| {
            issues
                .first()
                .map_or_else(|| "no such face".to_owned(), |i| i.message.clone())
        })
}

/// Whether two faces carry the same names and style, as with duplicate installs.
fn same_font(a: &FontFace, b: &FontFace) -> bool {
    a.weight == b.weight && a.italic == b.italic && a.name_records == b.name_records
}

fn strip_suffix(key: &str) -> Option<&str> {
    SMART_SUFFIXES
        .iter()
        .find_map(|suffix| key.strip_suffix(suffix))
        .filter(|base| !base.is_empty())
}

/// Resolve missing references again without an encoding suffix. Matches keep
/// the original reference (and therefore the suffixed name used for rewriting).
fn retry_without_suffix(index: &FontIndex, options: ResolveOptions, report: &mut ResolveReport) {
    let mut still_missing = Vec::new();
    for resolution in std::mem::take(&mut report.missing) {
        let key = normalize_name(&resolution.reference.name);
        let Some(base) = strip_suffix(&key) else {
            still_missing.push(resolution);
            continue;
        };
        let mut retry = resolution.reference.clone();
        retry.name = base.to_owned();
        let mut retried = index.resolve_with_options(std::slice::from_ref(&retry), options);
        let found = retried
            .resolved
            .pop()
            .map(|r| (r, false))
            .or_else(|| retried.ambiguous.pop().map(|r| (r, true)));
        match found {
            Some((mut found, ambiguous)) => {
                found.reference = resolution.reference;
                if ambiguous {
                    report.ambiguous.push(found);
                } else {
                    report.resolved.push(found);
                }
            }
            None => still_missing.push(resolution),
        }
    }
    report.missing = still_missing;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::SubtitleInput;

    #[test]
    fn unusable_override_is_reported_and_name_stays_missing() {
        let dir = tempfile::tempdir().unwrap();
        let ass = dir.path().join("a.ass");
        std::fs::write(
            &ass,
            "[V4+ Styles]\nFormat: Name, Fontname\nStyle: Default,Nowhere Sans\n[Events]\nFormat: Style, Text\nDialogue: Default,Hi\n",
        )
        .unwrap();
        let episode = Episode {
            mkv: dir.path().join("a.mkv"),
            subtitles: vec![SubtitleInput {
                path: ass,
                language: "en".into(),
            }],
        };
        let overrides = FontOverrides::from([(
            normalize_name("Nowhere Sans"),
            FaceRef {
                path: dir.path().join("missing.ttf"),
                face_index: 0,
            },
        )]);
        let plan = analyze(
            &episode,
            &FontIndex::new([]),
            &Settings::default(),
            &overrides,
        );
        assert!(
            plan.warnings
                .iter()
                .any(|w| w.contains("override for \"nowhere sans\" ignored"))
        );
        assert_eq!(plan.missing.len(), 1);
        assert!(plan.fonts.is_empty());
    }
}
