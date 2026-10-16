//! Conversion of ASS/SSA scripts to Matroska `S_TEXT/ASS` / `S_TEXT/SSA` tracks.
//!
//! CodecPrivate holds the script with event lines removed (the `[Events]`
//! header and its `Format:` line are kept). Each `Dialogue:` becomes a block
//! `ReadOrder,Layer,Style,Name,MarginL,MarginR,MarginV,Effect,Text` with the
//! start time as its timestamp and the end time as its duration. `Comment:`
//! and other event lines stay in CodecPrivate after `Format:`, as in mkvmerge.

use std::io;

use super::ebml::invalid;

const DEFAULT_FORMAT: &str =
    "Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text";

#[derive(Debug)]
pub struct AssTrack {
    pub codec_id: &'static str,
    pub codec_private: Vec<u8>,
    /// Sorted by start time, then source order.
    pub events: Vec<AssEvent>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AssEvent {
    pub start_ns: u64,
    pub duration_ns: u64,
    pub data: Vec<u8>,
}

pub fn convert(text: &str) -> io::Result<AssTrack> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut private = String::with_capacity(text.len().min(64 * 1024));
    let mut events = Vec::new();
    let mut section = String::new();
    let mut ssa = false;
    let mut format: Option<Vec<String>> = None;
    let mut saw_events = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed.to_ascii_lowercase();
            if section == "[v4 styles]" {
                ssa = true;
            }
            if section == "[events]" {
                saw_events = true;
            }
            push_line(&mut private, trimmed);
            continue;
        }
        if section == "[script info]"
            && let Some(value) = key_value(trimmed, "scripttype")
        {
            ssa |= value.eq_ignore_ascii_case("v4.00");
        }
        if section != "[events]" {
            push_line(&mut private, line);
            continue;
        }
        let Some((kind, value)) = trimmed.split_once(':') else {
            continue;
        };
        match kind.trim().to_ascii_lowercase().as_str() {
            "format" => {
                format = Some(
                    value
                        .split(',')
                        .map(|f| f.trim().to_ascii_lowercase())
                        .collect(),
                );
                push_line(&mut private, &format!("Format: {}", value.trim()));
            }
            "dialogue" => {
                let fields = format.get_or_insert_with(|| {
                    push_line(&mut private, &format!("Format: {DEFAULT_FORMAT}"));
                    DEFAULT_FORMAT
                        .split(',')
                        .map(|f| f.trim().to_ascii_lowercase())
                        .collect()
                });
                if let Some(event) = dialogue(value, fields, events.len()) {
                    events.push(event);
                }
            }
            // Comment and other non-rendered event lines stay in the header, as in mkvmerge.
            _ => push_line(&mut private, trimmed),
        }
    }
    if !saw_events {
        push_line(&mut private, "[Events]");
        push_line(&mut private, &format!("Format: {DEFAULT_FORMAT}"));
    } else if format.is_none() {
        push_line(&mut private, &format!("Format: {DEFAULT_FORMAT}"));
    }
    if section.is_empty() {
        return Err(invalid("not an ASS/SSA script: no sections"));
    }
    events.sort_by_key(|(order, e): &(usize, AssEvent)| (e.start_ns, *order));
    Ok(AssTrack {
        codec_id: if ssa { "S_TEXT/SSA" } else { "S_TEXT/ASS" },
        codec_private: private.into_bytes(),
        events: events.into_iter().map(|(_, e)| e).collect(),
    })
}

fn push_line(out: &mut String, line: &str) {
    out.push_str(line.trim_end_matches('\r'));
    out.push_str("\r\n");
}

fn key_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let (k, v) = line.split_once(':')?;
    k.trim().eq_ignore_ascii_case(key).then_some(v.trim())
}

fn dialogue(value: &str, format: &[String], read_order: usize) -> Option<(usize, AssEvent)> {
    if format.last().map(String::as_str) != Some("text") {
        return None;
    }
    let values: Vec<&str> = value.trim_start().splitn(format.len(), ',').collect();
    if values.len() != format.len() {
        return None;
    }
    let field = |name: &str| format.iter().position(|f| f == name).map(|i| values[i]);
    let start = parse_time(field("start")?)?;
    let end = parse_time(field("end")?)?;
    let layer = field("layer")
        .or_else(|| field("marked"))
        .unwrap_or("0")
        .trim();
    let data = format!(
        "{read_order},{layer},{},{},{},{},{},{},{}",
        field("style").unwrap_or("Default").trim(),
        field("name").unwrap_or("").trim(),
        field("marginl").unwrap_or("0").trim(),
        field("marginr").unwrap_or("0").trim(),
        field("marginv").unwrap_or("0").trim(),
        field("effect").unwrap_or("").trim(),
        field("text").unwrap_or(""),
    );
    Some((
        read_order,
        AssEvent {
            start_ns: start,
            duration_ns: end.saturating_sub(start),
            data: data.into_bytes(),
        },
    ))
}

/// Parse `H:MM:SS.cc` (any number of fractional digits) into nanoseconds.
fn parse_time(text: &str) -> Option<u64> {
    let mut parts = text.trim().split(':');
    let (h, m, s) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    let (sec, frac) = s.split_once('.').unwrap_or((s, ""));
    let digits: String = frac.chars().take(9).collect();
    let frac_ns = if digits.is_empty() {
        0
    } else {
        digits.parse::<u64>().ok()? * 10u64.pow(9 - digits.len() as u32)
    };
    let seconds = h.trim().parse::<u64>().ok()? * 3600
        + m.trim().parse::<u64>().ok()? * 60
        + sec.trim().parse::<u64>().ok()?;
    Some(seconds * 1_000_000_000 + frac_ns)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCRIPT: &str = "\u{feff}[Script Info]\r\nScriptType: v4.00+\r\n\r\n[V4+ Styles]\r\nFormat: Name, Fontname\r\nStyle: Default,Arial\r\n\r\n[Events]\r\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\r\nDialogue: 0,0:00:05.00,0:00:06.50,Default,,0,0,0,,Second, with comma\r\nComment: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,note\r\nDialogue: 1,0:00:01.00,0:00:02.00,Default,Bob,10,20,30,fx,{\\b1}First\r\n";

    #[test]
    fn converts_events_and_header() {
        let track = convert(SCRIPT).unwrap();
        assert_eq!(track.codec_id, "S_TEXT/ASS");
        let private = String::from_utf8(track.codec_private).unwrap();
        assert!(private.starts_with("[Script Info]\r\n"));
        assert!(private.ends_with(
            "Effect, Text\r\nComment: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,note\r\n"
        ));
        assert!(!private.contains("Dialogue"));
        assert_eq!(track.events.len(), 2);
        assert_eq!(track.events[0].start_ns, 1_000_000_000);
        assert_eq!(track.events[0].duration_ns, 1_000_000_000);
        assert_eq!(
            track.events[0].data,
            b"1,1,Default,Bob,10,20,30,fx,{\\b1}First"
        );
        assert_eq!(
            track.events[1].data,
            b"0,0,Default,,0,0,0,,Second, with comma"
        );
        assert_eq!(track.events[1].duration_ns, 1_500_000_000);
    }

    #[test]
    fn honors_custom_format_order() {
        let script = "[Script Info]\n[Events]\nFormat: Start, End, Style, Text\nDialogue: 0:00:00.5,0:00:01.25,S,Hi\n";
        let track = convert(script).unwrap();
        assert_eq!(track.events[0].start_ns, 500_000_000);
        assert_eq!(track.events[0].duration_ns, 750_000_000);
        assert_eq!(track.events[0].data, b"0,0,S,,0,0,0,,Hi");
    }

    #[test]
    fn detects_ssa() {
        let track = convert("[Script Info]\nScriptType: v4.00\n[V4 Styles]\n").unwrap();
        assert_eq!(track.codec_id, "S_TEXT/SSA");
        assert!(track.events.is_empty());
    }

    #[test]
    fn parses_times() {
        assert_eq!(parse_time("1:02:03.45"), Some(3_723_450_000_000));
        assert_eq!(parse_time("0:00:00"), Some(0));
        assert_eq!(parse_time("bad"), None);
    }
}
