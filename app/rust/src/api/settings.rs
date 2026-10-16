//! Settings shared with the CLI (`config.toml` in the per-user config directory).

use mfm_core::{LanguageRule, LanguageRules, MuxerKind, OutputMode, Settings};
use std::path::PathBuf;

/// Flat, Dart-friendly mirror of [`Settings`].
#[derive(Debug, Clone)]
pub struct SettingsDto {
    pub font_dirs: Vec<String>,
    /// `true` for the built-in muxer, `false` for mkvmerge.
    pub native_muxer: bool,
    pub mkvmerge: Option<String>,
    pub smart_match: bool,
    pub nearest_weight: bool,
    pub allow_synthesis: bool,
    pub subset: bool,
    pub rename: bool,
    pub overwrite: bool,
    pub output_dir_name: String,
    pub keep_temp: bool,
    pub ignored_fonts: Vec<String>,
    pub fallback_language: String,
    pub language_rules: Vec<LanguageRuleDto>,
}

#[derive(Debug, Clone)]
pub struct LanguageRuleDto {
    pub code: String,
    pub keywords: Vec<String>,
}

impl From<Settings> for SettingsDto {
    fn from(s: Settings) -> Self {
        let (overwrite, output_dir_name) = match s.output {
            OutputMode::Overwrite => (true, "output".to_owned()),
            OutputMode::Subdirectory { name } => (false, name),
        };
        Self {
            font_dirs: s
                .font_dirs
                .iter()
                .map(|p| p.display().to_string())
                .collect(),
            native_muxer: s.muxer == MuxerKind::Native,
            mkvmerge: s.mkvmerge.map(|p| p.display().to_string()),
            smart_match: s.smart_match,
            nearest_weight: s.nearest_weight,
            allow_synthesis: s.allow_synthesis,
            subset: s.subset,
            rename: s.rename,
            overwrite,
            output_dir_name,
            keep_temp: s.keep_temp,
            ignored_fonts: s.ignored_fonts,
            fallback_language: s.languages.fallback,
            language_rules: s
                .languages
                .rules
                .into_iter()
                .map(|r| LanguageRuleDto {
                    code: r.code,
                    keywords: r.keywords,
                })
                .collect(),
        }
    }
}

impl From<SettingsDto> for Settings {
    fn from(d: SettingsDto) -> Self {
        Self {
            font_dirs: d
                .font_dirs
                .into_iter()
                .filter(|s| !s.trim().is_empty())
                .map(PathBuf::from)
                .collect(),
            muxer: if d.native_muxer {
                MuxerKind::Native
            } else {
                MuxerKind::Mkvmerge
            },
            mkvmerge: d
                .mkvmerge
                .filter(|s| !s.trim().is_empty())
                .map(PathBuf::from),
            smart_match: d.smart_match,
            nearest_weight: d.nearest_weight,
            allow_synthesis: d.allow_synthesis,
            subset: d.subset,
            rename: d.rename,
            output: if d.overwrite {
                OutputMode::Overwrite
            } else {
                OutputMode::Subdirectory {
                    name: if d.output_dir_name.trim().is_empty() {
                        "output".into()
                    } else {
                        d.output_dir_name
                    },
                }
            },
            keep_temp: d.keep_temp,
            ignored_fonts: d.ignored_fonts,
            languages: LanguageRules {
                rules: d
                    .language_rules
                    .into_iter()
                    .map(|r| LanguageRule {
                        code: r.code,
                        keywords: r.keywords,
                    })
                    .collect(),
                fallback: d.fallback_language,
            },
        }
    }
}

fn settings_path() -> anyhow::Result<PathBuf> {
    mfm_core::config_dir()
        .map(|d| d.join("config.toml"))
        .ok_or_else(|| anyhow::anyhow!("cannot determine the configuration directory"))
}

/// Load the settings file; missing files yield the defaults.
pub fn load_settings() -> anyhow::Result<SettingsDto> {
    Ok(Settings::load(&settings_path()?)?.into())
}

pub fn save_settings(settings: SettingsDto) -> anyhow::Result<()> {
    Settings::from(settings).save(&settings_path()?)?;
    Ok(())
}

#[flutter_rust_bridge::frb(sync)]
pub fn default_settings() -> SettingsDto {
    Settings::default().into()
}

/// Font directories used when the list is empty.
#[flutter_rust_bridge::frb(sync)]
pub fn system_font_dirs() -> Vec<String> {
    mfm_core::system_font_dirs()
        .iter()
        .map(|p| p.display().to_string())
        .collect()
}

/// The mkvmerge that would be used, if any.
pub fn locate_mkvmerge(explicit: Option<String>) -> Option<String> {
    let explicit = explicit.filter(|s| !s.trim().is_empty()).map(PathBuf::from);
    mfm_core::MkvmergeMuxer::locate(explicit.as_deref()).map(|m| m.program().display().to_string())
}
