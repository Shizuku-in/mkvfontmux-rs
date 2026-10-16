use crate::language::LanguageRules;
use serde::Serialize;
use std::{
    io,
    path::{Path, PathBuf},
};

/// A Matroska file and the external subtitles to add to it.
#[derive(Debug, Clone, Serialize)]
pub struct Episode {
    pub mkv: PathBuf,
    pub subtitles: Vec<SubtitleInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubtitleInput {
    pub path: PathBuf,
    /// Language tag from the file name; may be edited before muxing.
    pub language: String,
}

/// Collect episodes from directories (top level only) and individual `.mkv` files.
///
/// A subtitle belongs to an episode when it sits in the same directory, has an
/// `.ass`/`.ssa` extension, and its name starts with the video's stem followed
/// by a non-alphanumeric character (so `Ep 1` does not claim `Ep 10.ass`).
/// Episodes are sorted by path; their subtitles by file name.
pub fn discover(inputs: &[PathBuf], languages: &LanguageRules) -> io::Result<Vec<Episode>> {
    let mut mkvs = Vec::new();
    for input in inputs {
        if input.is_dir() {
            for entry in std::fs::read_dir(input)? {
                let path = entry?.path();
                if path.is_file() && has_extension(&path, &["mkv"]) {
                    mkvs.push(path);
                }
            }
        } else if has_extension(input, &["mkv"]) {
            mkvs.push(input.clone());
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("not a directory or .mkv file: {}", input.display()),
            ));
        }
    }
    mkvs.sort();
    mkvs.dedup();

    let mut episodes = Vec::with_capacity(mkvs.len());
    for mkv in mkvs {
        let subtitles = find_subtitles(&mkv)?
            .into_iter()
            .map(|path| {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                SubtitleInput {
                    language: languages.detect(&name).to_owned(),
                    path,
                }
            })
            .collect();
        episodes.push(Episode { mkv, subtitles });
    }
    Ok(episodes)
}

fn find_subtitles(mkv: &Path) -> io::Result<Vec<PathBuf>> {
    let dir = mkv
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let stem = mkv
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if !path.is_file() || !has_extension(&path, &["ass", "ssa"]) {
            continue;
        }
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        let belongs = name
            .strip_prefix(&stem)
            .and_then(|rest| rest.chars().next())
            .is_some_and(|next| !next.is_alphanumeric());
        if belongs {
            found.push(path);
        }
    }
    found.sort_by_cached_key(|p| {
        p.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase()
    });
    Ok(found)
}

fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| extensions.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_subtitles_by_stem() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "Ep 1.mkv",
            "Ep 10.mkv",
            "Ep 1.sc.ass",
            "Ep 1.TC.ASS",
            "Ep 10.sc.ass",
            "Ep 1.txt",
        ] {
            std::fs::write(dir.path().join(name), b"").unwrap();
        }
        let episodes = discover(&[dir.path().to_owned()], &LanguageRules::default()).unwrap();
        assert_eq!(episodes.len(), 2);
        let ep1 = &episodes[0];
        assert!(ep1.mkv.ends_with("Ep 1.mkv"));
        let langs: Vec<_> = ep1.subtitles.iter().map(|s| s.language.as_str()).collect();
        assert_eq!(langs, ["zh-Hans", "zh-Hant"]);
        assert_eq!(episodes[1].subtitles.len(), 1);
    }
}
