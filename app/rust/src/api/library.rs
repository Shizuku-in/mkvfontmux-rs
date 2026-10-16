//! The font index ("library") as an opaque handle for Dart.

use super::settings::SettingsDto;
use flutter_rust_bridge::{DartFnFuture, frb};
use mfm_core::{Event, FontIndex, Reporter, Settings, normalize_name};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

#[frb(opaque)]
pub struct FontLibrary {
    pub(crate) index: FontIndex,
    stats: LibraryStats,
}

#[derive(Debug, Clone)]
pub struct LibraryStats {
    pub files: u32,
    pub faces: u32,
    pub cached: u32,
    pub rescanned: u32,
    /// Font files or faces that could not be read.
    pub issues: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IndexProgress {
    pub scanned: u32,
    pub to_scan: u32,
}

/// A face for display and manual selection.
#[derive(Debug, Clone)]
pub struct FaceInfo {
    pub path: String,
    pub face_index: u32,
    pub family: String,
    pub names: Vec<String>,
    pub weight: u16,
    pub italic: bool,
}

impl From<&mfm_core::FontFace> for FaceInfo {
    fn from(f: &mfm_core::FontFace) -> Self {
        Self {
            path: f.path.display().to_string(),
            face_index: f.face_index,
            family: f
                .family_names
                .first()
                .or(f.names.first())
                .cloned()
                .unwrap_or_else(|| {
                    f.path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned()
                }),
            names: f.names.clone(),
            weight: f.weight,
            italic: f.italic,
        }
    }
}

/// Forwards index progress to Dart at most every 100 ms.
struct Progress<F: Fn(IndexProgress) + Sync> {
    send: F,
    last: Mutex<Instant>,
}

impl<F: Fn(IndexProgress) + Sync> Reporter for Progress<F> {
    fn report(&self, event: Event) {
        if let Event::IndexProgress {
            scanned, to_scan, ..
        } = event
        {
            let mut last = self.last.lock().unwrap();
            if scanned == to_scan || last.elapsed() >= Duration::from_millis(100) {
                *last = Instant::now();
                (self.send)(IndexProgress {
                    scanned: scanned as u32,
                    to_scan: to_scan as u32,
                });
            }
        }
    }
}

impl FontLibrary {
    /// Scan the configured font directories, reusing the shared index cache.
    pub async fn build(
        settings: SettingsDto,
        on_progress: impl Fn(IndexProgress) -> DartFnFuture<()> + Send + Sync + 'static,
    ) -> FontLibrary {
        let settings = Settings::from(settings);
        let reporter = Progress {
            send: |p| {
                // Fire and forget: progress is advisory.
                drop(on_progress(p));
            },
            last: Mutex::new(Instant::now() - Duration::from_secs(1)),
        };
        let cache = mfm_core::cache_path();
        let (index, stats) = mfm_core::build_font_index(&settings, cache.as_deref(), &reporter);
        FontLibrary {
            index,
            stats: LibraryStats {
                files: stats.files as u32,
                faces: stats.faces as u32,
                cached: stats.cache_hits as u32,
                rescanned: stats.rescanned as u32,
                issues: stats
                    .issues
                    .iter()
                    .map(|i| format!("{}: {}", i.path.display(), i.message))
                    .collect(),
            },
        }
    }

    #[frb(sync, getter)]
    pub fn stats(&self) -> LibraryStats {
        self.stats.clone()
    }

    /// Faces whose names contain `query` (normalized), up to `limit`.
    #[frb(sync)]
    pub fn search(&self, query: String, limit: u32) -> Vec<FaceInfo> {
        let query = normalize_name(&query);
        self.index
            .faces()
            .iter()
            .filter(|f| {
                query.is_empty() || f.names.iter().any(|n| normalize_name(n).contains(&query))
            })
            .take(limit as usize)
            .map(FaceInfo::from)
            .collect()
    }
}
