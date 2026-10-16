use crate::{MuxError, MuxJob, MuxObserver, MuxOutcome, Muxer};
use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
    time::Duration,
};

/// Backend that runs MKVToolNix's `mkvmerge`.
#[derive(Debug, Clone)]
pub struct MkvmergeMuxer {
    program: PathBuf,
}

impl MkvmergeMuxer {
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }

    pub fn program(&self) -> &Path {
        &self.program
    }

    /// Find `mkvmerge`: `explicit` path, `MKVMERGE_BIN`, `PATH`, then the usual
    /// install locations.
    pub fn locate(explicit: Option<&Path>) -> Option<Self> {
        let exe = if cfg!(windows) {
            "mkvmerge.exe"
        } else {
            "mkvmerge"
        };
        let mut candidates: Vec<PathBuf> = Vec::new();
        candidates.extend(explicit.map(Path::to_path_buf));
        candidates.extend(std::env::var_os("MKVMERGE_BIN").map(PathBuf::from));
        if let Some(path) = std::env::var_os("PATH") {
            candidates.extend(std::env::split_paths(&path).map(|dir| dir.join(exe)));
        }
        if cfg!(windows) {
            for var in ["ProgramFiles", "ProgramFiles(x86)"] {
                if let Some(dir) = std::env::var_os(var) {
                    candidates.push(PathBuf::from(dir).join("MKVToolNix").join(exe));
                }
            }
        } else if cfg!(target_os = "macos") {
            if let Ok(apps) = std::fs::read_dir("/Applications") {
                for app in apps.flatten() {
                    if app.file_name().to_string_lossy().starts_with("MKVToolNix") {
                        candidates.push(app.path().join("Contents/MacOS").join(exe));
                    }
                }
            }
            candidates.push("/opt/homebrew/bin/mkvmerge".into());
            candidates.push("/usr/local/bin/mkvmerge".into());
        }
        candidates.into_iter().find(|p| p.is_file()).map(Self::new)
    }

    /// Arguments after the global options; passed through an options file so
    /// that many attachments cannot exceed the command-line length limit.
    fn arguments(job: &MuxJob) -> Vec<String> {
        let mut args = vec!["-o".to_owned(), path_arg(&job.output), path_arg(&job.input)];
        for track in &job.subtitles {
            args.extend(["--language".into(), format!("0:{}", track.language)]);
            if let Some(name) = &track.name {
                args.extend(["--track-name".into(), format!("0:{name}")]);
            }
            args.extend([
                "--default-track-flag".into(),
                format!("0:{}", if track.default { "yes" } else { "no" }),
            ]);
            args.push(path_arg(&track.path));
        }
        for attachment in &job.attachments {
            args.extend([
                "--attachment-name".into(),
                attachment.name.clone(),
                "--attachment-mime-type".into(),
                attachment.mime_type.clone(),
                "--attach-file".into(),
                path_arg(&attachment.path),
            ]);
        }
        args
    }
}

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

impl Muxer for MkvmergeMuxer {
    fn name(&self) -> &'static str {
        "mkvmerge"
    }

    fn mux(&self, job: &MuxJob, observer: &mut dyn MuxObserver) -> Result<MuxOutcome, MuxError> {
        let options_file = job.output.with_extension("mkvmerge.json");
        let args = Self::arguments(job);
        std::fs::write(
            &options_file,
            serde_json::to_vec(&args).map_err(std::io::Error::other)?,
        )?;
        let result = self.run(job, &options_file, observer);
        let _ = std::fs::remove_file(&options_file);
        if result.is_err() {
            let _ = std::fs::remove_file(&job.output);
        }
        result
    }
}

impl MkvmergeMuxer {
    fn run(
        &self,
        job: &MuxJob,
        options_file: &Path,
        observer: &mut dyn MuxObserver,
    ) -> Result<MuxOutcome, MuxError> {
        tracing::debug!(program = %self.program.display(), input = %job.input.display(), "running mkvmerge");
        let mut command = Command::new(&self.program);
        command
            .args(["--gui-mode", "--ui-language", "en"])
            .arg(format!("@{}", options_file.display()))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            // In GUI mode mkvmerge reports errors on stdout; an unread stderr pipe could block it.
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = command.spawn().map_err(|source| MuxError::Spawn {
            program: self.program.clone(),
            source,
        })?;

        // Read stdout on a thread so the main loop can poll for cancellation.
        let stdout = child.stdout.take().expect("stdout is piped");
        let (tx, rx) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        let mut outcome = MuxOutcome::default();
        let mut errors = Vec::new();
        loop {
            if observer.cancelled() {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                return Err(MuxError::Cancelled);
            }
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(line) => {
                    if let Some(rest) = line.strip_prefix("#GUI#progress ") {
                        if let Ok(percent) = rest.trim_end_matches('%').trim().parse::<u8>() {
                            observer.progress(percent.min(100));
                        }
                    } else if let Some(rest) = line.strip_prefix("#GUI#warning ") {
                        outcome.warnings.push(rest.to_owned());
                    } else if let Some(rest) = line.strip_prefix("#GUI#error ") {
                        errors.push(rest.to_owned());
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        let _ = reader.join();
        let status = child.wait()?;
        // mkvmerge: 0 = success, 1 = success with warnings, 2 = error.
        match status.code() {
            Some(0 | 1) => Ok(outcome),
            code => {
                if errors.is_empty() {
                    errors.push(format!("exited with {code:?}"));
                }
                Err(MuxError::Failed {
                    backend: "mkvmerge",
                    message: errors.join("; "),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Attachment, SubtitleTrack};

    #[test]
    fn builds_track_and_attachment_arguments() {
        let job = MuxJob {
            input: "in.mkv".into(),
            output: "out.mkv".into(),
            subtitles: vec![SubtitleTrack {
                path: "a.sc.ass".into(),
                language: "zh-Hans".into(),
                name: None,
                default: true,
            }],
            attachments: vec![Attachment {
                path: "tmp/x.ttf".into(),
                name: "x.ttf".into(),
                mime_type: "application/x-truetype-font".into(),
            }],
        };
        let args = MkvmergeMuxer::arguments(&job);
        assert_eq!(&args[..3], ["-o", "out.mkv", "in.mkv"]);
        assert!(args.windows(2).any(|w| w == ["--language", "0:zh-Hans"]));
        assert!(args.windows(2).any(|w| w == ["--attachment-name", "x.ttf"]));
        assert_eq!(args.last().map(String::as_str), Some("tmp/x.ttf"));
    }
}
