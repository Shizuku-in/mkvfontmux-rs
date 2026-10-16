use ass_fonts::{FontFace, FontIndex, FontName, NameKind, SCAN_FORMAT_VERSION, ScanIssue};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::UNIX_EPOCH,
};

const MAGIC: [u8; 4] = *b"MFMX";
/// Bump when the cache layout or the fallback alias rules change.
const CACHE_VERSION: u32 = 1;

/// Persistent per-file scan results keyed by canonical path.
///
/// A file is rescanned when its size or modification time changes, or when
/// `ass-fonts` reports a different [`SCAN_FORMAT_VERSION`].
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FontCache {
    files: BTreeMap<PathBuf, CachedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedFile {
    size: u64,
    modified_ns: u128,
    faces: Vec<FontFace>,
    issues: Vec<ScanIssue>,
}

#[derive(Serialize, Deserialize)]
struct Header {
    magic: [u8; 4],
    cache_version: u32,
    scan_version: u32,
}

impl FontCache {
    /// Load a cache file; a missing, corrupt, or outdated file yields an empty cache.
    pub fn load(path: &Path) -> Self {
        let Ok(bytes) = fs::read(path) else {
            return Self::default();
        };
        let parsed = postcard::take_from_bytes::<Header>(&bytes)
            .ok()
            .and_then(|(header, rest)| {
                (header.magic == MAGIC
                    && header.cache_version == CACHE_VERSION
                    && header.scan_version == SCAN_FORMAT_VERSION)
                    .then(|| postcard::from_bytes::<Self>(rest).ok())
                    .flatten()
            });
        if parsed.is_none() {
            tracing::info!(path = %path.display(), "font cache missing or outdated; rebuilding");
        }
        parsed.unwrap_or_default()
    }

    /// Write the cache atomically (temporary file, then rename).
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let header = Header {
            magic: MAGIC,
            cache_version: CACHE_VERSION,
            scan_version: SCAN_FORMAT_VERSION,
        };
        let mut bytes = postcard::to_stdvec(&header).map_err(io::Error::other)?;
        bytes.extend(postcard::to_stdvec(self).map_err(io::Error::other)?);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, bytes)?;
        fs::rename(&tmp, path)
    }

    /// Number of cached font files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// Progress of [`build_index`], reported after each rescanned file.
#[derive(Debug, Clone, Copy)]
pub struct IndexProgress {
    pub scanned: usize,
    pub to_scan: usize,
    pub cached: usize,
}

/// Summary of an index build.
#[derive(Debug, Default)]
pub struct IndexStats {
    pub files: usize,
    pub cache_hits: usize,
    pub rescanned: usize,
    pub faces: usize,
    pub issues: Vec<ScanIssue>,
}

/// Discover font files below `roots`, reuse unchanged entries from `cache`,
/// scan the rest in parallel, and build a [`FontIndex`].
///
/// Entries for files that no longer exist under `roots` are dropped from the
/// cache. Faces without any decodable name get aliases derived from the file
/// name, matching the original tool's fallback.
pub fn build_index<I, P>(
    roots: I,
    cache: &mut FontCache,
    progress: impl Fn(IndexProgress) + Sync,
) -> (FontIndex, IndexStats)
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let (paths, mut issues) = ass_fonts::discover_font_files(roots);
    let paths: Vec<PathBuf> = paths.into_iter().map(simplify_path).collect();
    let mut stats = IndexStats {
        files: paths.len(),
        ..Default::default()
    };

    let mut fresh = BTreeMap::new();
    let mut stale = Vec::new();
    for path in paths {
        let Some((size, modified_ns)) = stat(&path) else {
            stale.push((path, None));
            continue;
        };
        match cache.files.remove(&path) {
            Some(entry) if entry.size == size && entry.modified_ns == modified_ns => {
                fresh.insert(path, entry);
            }
            _ => stale.push((path, Some((size, modified_ns)))),
        }
    }
    stats.cache_hits = fresh.len();
    stats.rescanned = stale.len();

    let done = AtomicUsize::new(0);
    let to_scan = stale.len();
    let cached = fresh.len();
    let scanned: Vec<_> = stale
        .into_par_iter()
        .map(|(path, meta)| {
            let (mut faces, issues) = ass_fonts::scan_font_file(&path);
            add_filename_aliases(&path, &mut faces);
            let completed = done.fetch_add(1, Ordering::Relaxed) + 1;
            progress(IndexProgress {
                scanned: completed,
                to_scan,
                cached,
            });
            (path, meta, faces, issues)
        })
        .collect();
    for (path, meta, faces, file_issues) in scanned {
        // Without metadata the entry can never match again, so it is rescanned next time.
        let (size, modified_ns) = meta.unwrap_or((u64::MAX, 0));
        let entry = CachedFile {
            size,
            modified_ns,
            faces,
            issues: file_issues,
        };
        fresh.insert(path, entry);
    }

    for entry in fresh.values() {
        issues.extend(entry.issues.iter().cloned());
    }
    let faces: Vec<FontFace> = fresh
        .values()
        .flat_map(|e| e.faces.iter().cloned())
        .collect();
    stats.faces = faces.len();
    stats.issues = issues;
    cache.files = fresh;
    (FontIndex::new(faces), stats)
}

/// Drop the `\\?\` verbatim prefix that canonicalization adds on Windows when
/// the remaining path is an ordinary drive path, so paths display normally
/// and work with tools that reject verbatim paths.
pub(crate) fn simplify_path(path: PathBuf) -> PathBuf {
    let Some(text) = path.to_str() else {
        return path;
    };
    match text.strip_prefix(r"\\?\") {
        Some(rest)
            if rest.len() < 260
                && rest.as_bytes().get(1) == Some(&b':')
                && rest.as_bytes().first().is_some_and(u8::is_ascii_alphabetic) =>
        {
            PathBuf::from(rest)
        }
        _ => path,
    }
}

fn stat(path: &Path) -> Option<(u64, u128)> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();
    Some((meta.len(), modified))
}

/// For faces without usable names, register the file stem and its `&`-separated
/// parts (with a trailing `P`/`(P)` proportional marker also stripped) as aliases.
fn add_filename_aliases(path: &Path, faces: &mut [FontFace]) {
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()).map(str::trim) else {
        return;
    };
    if stem.is_empty() {
        return;
    }
    for face in faces.iter_mut().filter(|f| f.names.is_empty()) {
        let mut aliases = vec![stem.to_owned()];
        for part in stem
            .split(['&', '＆'])
            .map(str::trim)
            .filter(|p| !p.is_empty())
        {
            aliases.push(part.to_owned());
            let base = part
                .strip_suffix("(P)")
                .or_else(|| part.strip_suffix('P'))
                .or_else(|| part.strip_suffix('p'))
                .map(str::trim);
            if let Some(base) = base.filter(|b| !b.is_empty()) {
                aliases.push(base.to_owned());
            }
        }
        aliases.sort();
        aliases.dedup();
        for alias in aliases {
            face.name_records.push(FontName {
                name: alias.clone(),
                kind: NameKind::InternalName,
                name_id: None,
            });
            face.names.push(alias);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_aliases_only_for_nameless_faces() {
        let face = |names: Vec<String>| FontFace {
            path: "x".into(),
            face_index: 0,
            names,
            family_names: vec![],
            name_records: vec![],
            weight: 400,
            italic: false,
        };
        let mut faces = vec![face(vec![]), face(vec!["Named".into()])];
        add_filename_aliases(Path::new("/f/DFKai & DFPKai.ttc"), &mut faces);
        assert_eq!(faces[0].names, ["DFKai", "DFKai & DFPKai", "DFPKai"]);
        assert_eq!(faces[1].names, ["Named"]);
    }

    #[test]
    fn simplifies_verbatim_drive_paths() {
        assert_eq!(
            simplify_path(r"\\?\C:\Fonts\a.ttf".into()),
            PathBuf::from(r"C:\Fonts\a.ttf")
        );
        assert_eq!(
            simplify_path(r"\\?\UNC\srv\a.ttf".into()),
            PathBuf::from(r"\\?\UNC\srv\a.ttf")
        );
        assert_eq!(
            simplify_path("/usr/a.ttf".into()),
            PathBuf::from("/usr/a.ttf")
        );
    }

    #[test]
    fn cache_round_trip_and_version_check() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.bin");
        let mut cache = FontCache::default();
        cache.files.insert(
            "a.ttf".into(),
            CachedFile {
                size: 1,
                modified_ns: 2,
                faces: vec![],
                issues: vec![],
            },
        );
        cache.save(&path).unwrap();
        assert_eq!(FontCache::load(&path).len(), 1);
        fs::write(&path, b"garbage").unwrap();
        assert!(FontCache::load(&path).is_empty());
    }
}
