//! The mkvfontmux pipeline, shared by the CLI and the Flutter GUI.
//!
//! 1. [`build_font_index`] scans font directories with a persistent cache.
//! 2. [`discover`] pairs `.mkv` files with external `.ass`/`.ssa` subtitles.
//! 3. [`analyze`] resolves subtitle fonts into an [`EpisodePlan`] (preflight).
//! 4. [`execute`] subsets and renames fonts, rewrites subtitles, and muxes.

mod analyze;
mod discover;
mod events;
mod execute;
mod language;
mod settings;

pub use analyze::{
    EpisodePlan, FontUse, GlyphGap, MissingFont, MissingKind, PreparedSubtitle, analyze,
};
pub use ass_fonts::{FontFace, FontIndex};
pub use discover::{Episode, SubtitleInput, discover};
pub use events::{Event, Reporter, Silent, Stage};
pub use execute::{EpisodeOutcome, ExecuteError, execute};
pub use language::{LanguageRule, LanguageRules};
pub use mfm_font::IndexStats;
pub use mfm_mkv::{MkvmergeMuxer, Muxer};
pub use settings::{OutputMode, Settings, cache_path, config_dir};

use std::path::Path;

/// Build the font index for `settings`, reusing and updating the cache at
/// `cache_path` when given.
pub fn build_font_index(
    settings: &Settings,
    cache_path: Option<&Path>,
    reporter: &dyn Reporter,
) -> (FontIndex, IndexStats) {
    let mut cache = cache_path
        .map(mfm_font::FontCache::load)
        .unwrap_or_default();
    let (index, stats) = mfm_font::build_index(settings.effective_font_dirs(), &mut cache, |p| {
        reporter.report(Event::IndexProgress {
            scanned: p.scanned,
            to_scan: p.to_scan,
            cached: p.cached,
        })
    });
    if let Some(path) = cache_path
        && let Err(e) = cache.save(path)
    {
        tracing::warn!("cannot save font cache {}: {e}", path.display());
    }
    (index, stats)
}
