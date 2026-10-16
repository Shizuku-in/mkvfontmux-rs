use crate::language::LanguageRules;
use serde::{Deserialize, Serialize};
use std::{
    io,
    path::{Path, PathBuf},
};

/// User settings shared by the CLI and the GUI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Settings {
    /// Font directories to index; empty means the platform font directories.
    pub font_dirs: Vec<PathBuf>,
    /// Explicit `mkvmerge` path; otherwise it is searched for.
    pub mkvmerge: Option<PathBuf>,
    /// Retry missing names without `_gbk`-style encoding suffixes.
    pub smart_match: bool,
    /// Pick the nearest available weight within a family.
    pub nearest_weight: bool,
    /// Accept regular faces for bold/italic requests that renderers synthesize.
    pub allow_synthesis: bool,
    /// Subset fonts; when off, original font files are attached unchanged.
    pub subset: bool,
    /// Give subset fonts random family names and rewrite the subtitles to match.
    pub rename: bool,
    pub output: OutputMode,
    /// Keep the temporary directory with subset fonts and rewritten subtitles.
    pub keep_temp: bool,
    /// Font names never embedded (compared after normalization).
    pub ignored_fonts: Vec<String>,
    pub languages: LanguageRules,
}

/// Where muxed files are written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "mode")]
pub enum OutputMode {
    /// A subdirectory next to each source file.
    Subdirectory { name: String },
    /// Replace the source file after a successful mux.
    Overwrite,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_dirs: Vec::new(),
            mkvmerge: None,
            smart_match: true,
            nearest_weight: true,
            allow_synthesis: true,
            subset: true,
            rename: true,
            output: OutputMode::Subdirectory {
                name: "output".into(),
            },
            keep_temp: false,
            ignored_fonts: vec!["Default".into(), "Arial".into(), "sans-serif".into()],
            languages: LanguageRules::default(),
        }
    }
}

impl Settings {
    /// Font directories to scan, falling back to the platform directories.
    pub fn effective_font_dirs(&self) -> Vec<PathBuf> {
        if self.font_dirs.is_empty() {
            mfm_font::system_font_dirs()
        } else {
            self.font_dirs.clone()
        }
    }

    /// Load TOML settings; a missing file yields the defaults.
    pub fn load(path: &Path) -> io::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                toml::from_str(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e),
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = toml::to_string_pretty(self).map_err(io::Error::other)?;
        std::fs::write(path, text)
    }

    /// Apply the keys of the original tool's `config.ini`
    /// (`mkvmerge-bin`, `font-directory` with `;` separators).
    pub fn apply_legacy_ini(&mut self, text: &str) {
        for line in text.lines().map(str::trim) {
            if line.starts_with(['#', ';']) {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim().trim_matches(['"', '\'']);
            if value.is_empty() {
                continue;
            }
            match key.trim().to_ascii_lowercase().as_str() {
                "mkvmerge-bin" => self.mkvmerge = Some(value.into()),
                "font-directory" => {
                    self.font_dirs = value
                        .split(';')
                        .map(str::trim)
                        .filter(|d| !d.is_empty())
                        .map(PathBuf::from)
                        .collect();
                }
                _ => {}
            }
        }
    }
}

/// Per-user directories for settings and the font cache.
pub fn config_dir() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    };
    base.map(|b| b.join("mkvfontmux"))
}

/// Location of the persistent font index cache.
pub fn cache_path() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Caches"))
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
    };
    base.map(|b| b.join("mkvfontmux").join("font-index.bin"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_round_trip() {
        let settings = Settings::default();
        let text = toml::to_string_pretty(&settings).unwrap();
        assert_eq!(toml::from_str::<Settings>(&text).unwrap(), settings);
        // Partial files fill in defaults.
        let partial: Settings = toml::from_str("subset = false").unwrap();
        assert!(!partial.subset && partial.rename);
        // A languages table with only a fallback keeps the default rules.
        let partial: Settings = toml::from_str("[languages]
fallback = \"ja\"").unwrap();
        assert_eq!(partial.languages.fallback, "ja");
        assert_eq!(partial.languages.rules, LanguageRules::default().rules);
    }

    #[test]
    fn legacy_ini() {
        let mut settings = Settings::default();
        settings.apply_legacy_ini("# c\nmkvmerge-bin=\"C:\\m\\mkvmerge.exe\"\nfont-directory=H:\\A; H:\\B\npyftsubset-bin=x\n");
        assert_eq!(settings.mkvmerge, Some(PathBuf::from(r"C:\m\mkvmerge.exe")));
        assert_eq!(
            settings.font_dirs,
            [PathBuf::from(r"H:\A"), PathBuf::from(r"H:\B")]
        );
    }
}
