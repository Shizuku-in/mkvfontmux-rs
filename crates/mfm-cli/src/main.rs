use anyhow::{Context, bail};
use clap::Parser;
use mfm_core::{
    EpisodePlan, Event, MkvmergeMuxer, OutputMode, Reporter, Settings, Stage, analyze,
    build_font_index, discover, execute,
};
use std::{
    io::Write,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Embed the fonts used by ASS subtitles into MKV files as subsets.
///
/// Each .mkv in the given directories (or each given .mkv) is muxed with the
/// .ass/.ssa files next to it whose names start with the video's name.
#[derive(Parser, Debug)]
#[command(name = "mkvfontmux", version)]
struct Cli {
    /// Directories or .mkv files.
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// mkvmerge executable.
    #[arg(short = 'm', long = "mkvmerge-bin", value_name = "PATH")]
    mkvmerge: Option<PathBuf>,
    /// Match font names exactly (no `_gbk`-style suffix fallback).
    #[arg(short = 'f', long)]
    force_match: bool,
    /// Font directories, separated by `;` or repeated.
    #[arg(
        short = 'd',
        long = "font-directory",
        value_name = "DIRS",
        value_delimiter = ';'
    )]
    font_dirs: Vec<PathBuf>,
    /// Attach original font files instead of subsets.
    #[arg(short = 'n', long)]
    disable_subset: bool,
    /// Keep original font names in subsets.
    #[arg(long)]
    no_rename: bool,
    /// Require exact weights instead of the nearest one in a family.
    #[arg(long)]
    exact_weight: bool,
    /// Do not accept regular faces for synthesized bold/italic.
    #[arg(long)]
    no_synthesis: bool,
    /// Write mux.log next to the first input.
    #[arg(short = 'l', long)]
    save_log: bool,
    /// Replace source files instead of writing to an `output` subdirectory.
    #[arg(short = 'o', long)]
    overwrite: bool,
    /// Keep temporary subset fonts and rewritten subtitles.
    #[arg(short = 'r', long)]
    save_temp: bool,
    /// Only report font matching; do not mux.
    #[arg(short = 'p', long = "only-print-matchfont")]
    report_only: bool,
    /// Language tag for subtitles whose names have no language keyword.
    #[arg(short = 's', long = "subtitle-language", value_name = "TAG")]
    fallback_language: Option<String>,
    /// Ignored; fonts are subset with the built-in HarfBuzz.
    #[arg(short = 'y', long = "pyftsubset-bin", value_name = "PATH", hide = true)]
    pyftsubset: Option<PathBuf>,
    /// Settings file (default: the per-user config.toml).
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
    /// Rescan all fonts instead of using the index cache.
    #[arg(long)]
    no_cache: bool,
    /// Print the analysis and results as JSON on stdout.
    #[arg(long)]
    json: bool,
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(true) => std::process::ExitCode::SUCCESS,
        Ok(false) => std::process::ExitCode::from(1),
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::ExitCode::from(2)
        }
    }
}

fn load_settings(cli: &Cli) -> anyhow::Result<Settings> {
    let path = cli
        .config
        .clone()
        .or_else(|| mfm_core::config_dir().map(|d| d.join("config.toml")));
    let mut settings = match &path {
        Some(path) => {
            Settings::load(path).with_context(|| format!("cannot load {}", path.display()))?
        }
        None => Settings::default(),
    };
    // The original tool kept config.ini next to the executable.
    if let Some(ini) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.join("config.ini")))
        && let Ok(text) = std::fs::read_to_string(&ini)
    {
        settings.apply_legacy_ini(&text);
    }
    if let Some(m) = &cli.mkvmerge {
        settings.mkvmerge = Some(m.clone());
    }
    if !cli.font_dirs.is_empty() {
        settings.font_dirs = cli.font_dirs.clone();
    }
    settings.smart_match &= !cli.force_match;
    settings.subset &= !cli.disable_subset;
    settings.rename &= !cli.no_rename;
    settings.nearest_weight &= !cli.exact_weight;
    settings.allow_synthesis &= !cli.no_synthesis;
    settings.keep_temp |= cli.save_temp;
    if cli.overwrite {
        settings.output = OutputMode::Overwrite;
    }
    if let Some(lang) = &cli.fallback_language {
        settings.languages.fallback = lang.clone();
    }
    Ok(settings)
}

fn init_logging(cli: &Cli) -> anyhow::Result<()> {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    let file_layer = if cli.save_log {
        let dir = cli.inputs.first().map(|p| {
            if p.is_dir() {
                p.clone()
            } else {
                p.parent().unwrap_or(p).to_owned()
            }
        });
        let path = dir.unwrap_or_default().join("mux.log");
        let file = std::fs::File::create(&path)
            .with_context(|| format!("cannot create {}", path.display()))?;
        Some(
            fmt::layer()
                .with_ansi(false)
                .with_writer(std::sync::Mutex::new(file))
                .with_filter(EnvFilter::new("info")),
        )
    } else {
        None
    };
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_filter(filter),
        )
        .with(file_layer)
        .init();
    Ok(())
}

struct Console {
    cancelled: Arc<AtomicBool>,
}

impl Reporter for Console {
    fn report(&self, event: Event) {
        match event {
            Event::IndexProgress {
                scanned, to_scan, ..
            } => {
                if scanned == to_scan || scanned % 50 == 0 {
                    eprint!("\rScanning fonts {scanned}/{to_scan}");
                    if scanned == to_scan {
                        eprintln!();
                    }
                }
            }
            Event::Stage { stage, .. } => {
                let text = match stage {
                    Stage::Subsetting => "subsetting fonts",
                    Stage::Rewriting => "rewriting subtitles",
                    Stage::Muxing => "muxing",
                    Stage::Finalizing => "finalizing",
                };
                eprintln!("  · {text}");
            }
            Event::FontReady {
                font,
                attachment,
                bytes,
                ..
            } => {
                eprintln!(
                    "    {font} → {attachment} ({:.1} KiB)",
                    bytes as f64 / 1024.0
                );
            }
            Event::MuxProgress { percent, .. } => {
                eprint!("\r    {percent:>3}%");
                if percent == 100 {
                    eprintln!();
                }
                let _ = std::io::stderr().flush();
            }
            Event::Warning { message, .. } => {
                tracing::warn!("{message}");
                eprintln!("  ! {message}");
            }
        }
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

fn print_plan(plan: &EpisodePlan) {
    for sub in &plan.subtitles {
        let name = sub.path.file_name().unwrap_or_default().to_string_lossy();
        eprintln!("  subtitle {name} [{}] ({})", sub.language, sub.encoding);
    }
    for font in &plan.fonts {
        let face = &font.face;
        let mut notes = Vec::new();
        if font.ambiguous {
            notes.push("ambiguous".to_owned());
        }
        if let Some(base) = &font.fallback_name {
            notes.push(format!("matched as {base}"));
        }
        let notes = if notes.is_empty() {
            String::new()
        } else {
            format!(" ({})", notes.join(", "))
        };
        eprintln!(
            "  ✓ {} → {}#{} [{} chars]{notes}",
            font.name,
            face.path.display(),
            face.face_index,
            font.characters.len()
        );
    }
    for missing in &plan.missing {
        let style = match (missing.weight >= 600, missing.italic) {
            (false, false) => "",
            (true, false) => " bold",
            (false, true) => " italic",
            (true, true) => " bold italic",
        };
        eprintln!("  ✗ missing: {}{style}", missing.name);
    }
    for gap in &plan.missing_glyphs {
        let chars: String = gap.characters.iter().collect();
        eprintln!(
            "  ! {} lacks {} glyphs: {chars}",
            gap.font,
            gap.characters.len()
        );
    }
    for warning in &plan.warnings {
        eprintln!("  ! {warning}");
    }
}

fn run(cli: Cli) -> anyhow::Result<bool> {
    init_logging(&cli)?;
    if cli.pyftsubset.is_some() {
        eprintln!("note: --pyftsubset-bin is ignored; fonts are subset with the built-in HarfBuzz");
    }
    let settings = load_settings(&cli)?;

    let episodes = discover(&cli.inputs, &settings.languages)?;
    if episodes.is_empty() {
        bail!("no .mkv files found");
    }
    let muxer = if cli.report_only {
        None
    } else {
        Some(MkvmergeMuxer::locate(settings.mkvmerge.as_deref()).context(
            "mkvmerge not found; pass --mkvmerge-bin, set MKVMERGE_BIN, or add MKVToolNix to PATH",
        )?)
    };

    let cancelled = Arc::new(AtomicBool::new(false));
    let console = Console {
        cancelled: cancelled.clone(),
    };
    let cache = if cli.no_cache {
        None
    } else {
        mfm_core::cache_path()
    };
    let (index, stats) = build_font_index(&settings, cache.as_deref(), &console);
    eprintln!(
        "Font index: {} faces in {} files ({} cached, {} scanned, {} issues)",
        stats.faces,
        stats.files,
        stats.cache_hits,
        stats.rescanned,
        stats.issues.len()
    );
    for issue in &stats.issues {
        tracing::info!("font scan: {}: {}", issue.path.display(), issue.message);
    }

    let mut ok = true;
    let mut json_results = Vec::new();
    for (i, episode) in episodes.iter().enumerate() {
        let name = episode
            .mkv
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        eprintln!("[{}/{}] {name}", i + 1, episodes.len());
        if episode.subtitles.is_empty() {
            eprintln!("  skipped: no matching subtitles");
            continue;
        }
        let plan = analyze(episode, &index, &settings);
        print_plan(&plan);
        let Some(muxer) = &muxer else {
            ok &= plan.missing.is_empty();
            json_results.push(serde_json::json!({ "plan": plan }));
            continue;
        };
        match execute(i, &plan, &settings, muxer, &console) {
            Ok(outcome) => {
                eprintln!("  done: {}", outcome.output.display());
                if let Some(temp) = &outcome.temp_dir {
                    eprintln!("  temporary files kept in {}", temp.display());
                }
                json_results.push(serde_json::json!({ "plan": plan, "outcome": outcome }));
            }
            Err(e) => {
                ok = false;
                eprintln!("  failed: {e}");
                json_results.push(serde_json::json!({ "plan": plan, "error": e.to_string() }));
            }
        }
    }
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&json_results)?);
    }
    Ok(ok)
}
