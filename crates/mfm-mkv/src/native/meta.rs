//! Rewriting of the small Segment children: EBML header, Info, Tracks, Attachments.

use super::ebml::{self, id};
use std::io;

const APP: &str = concat!("mkvfontmux ", env!("CARGO_PKG_VERSION"));

/// EBML header for the output. DocTypeVersion 4 is needed for LanguageBCP47.
pub fn ebml_header(doc_type_version: u64, doc_type_read_version: u64) -> Vec<u8> {
    let mut out = Vec::new();
    ebml::put_master(&mut out, id::EBML, |b| {
        ebml::put_uint(b, id::EBML_VERSION, 1);
        ebml::put_uint(b, id::EBML_READ_VERSION, 1);
        ebml::put_uint(b, id::EBML_MAX_ID_LENGTH, 4);
        ebml::put_uint(b, id::EBML_MAX_SIZE_LENGTH, 8);
        // Subtitle tracks and attachments are not valid WebM, so always write Matroska.
        ebml::put_str(b, id::DOC_TYPE, "matroska");
        ebml::put_uint(b, id::DOC_TYPE_VERSION, doc_type_version.max(4));
        ebml::put_uint(
            b,
            id::DOC_TYPE_READ_VERSION,
            doc_type_read_version.clamp(2, 4),
        );
    });
    out
}

/// Rewritten Info element and its TimestampScale.
pub fn info(data: &[u8]) -> io::Result<(Vec<u8>, u64)> {
    let children = ebml::children(data)?;
    let mut scale = 1_000_000;
    let mut out = Vec::new();
    ebml::put_master(&mut out, id::INFO, |b| {
        for child in children {
            match child.id {
                id::TIMESTAMP_SCALE => {
                    scale = ebml::read_uint(child.data).max(1);
                    b.extend_from_slice(child.raw);
                }
                id::CRC32 | id::VOID | id::WRITING_APP | id::MUXING_APP => {}
                _ => b.extend_from_slice(child.raw),
            }
        }
        ebml::put_str(b, id::MUXING_APP, APP);
        ebml::put_str(b, id::WRITING_APP, APP);
    });
    Ok((out, scale))
}

/// What the muxer needs to know about an existing track.
#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub number: u64,
    pub uid: u64,
    pub track_type: u64,
}

pub const TRACK_VIDEO: u64 = 1;
pub const TRACK_AUDIO: u64 = 2;
pub const TRACK_SUBTITLE: u64 = 17;

/// Existing TrackEntry elements (raw) and their parsed summaries.
pub fn tracks(data: &[u8]) -> io::Result<(Vec<u8>, Vec<TrackInfo>)> {
    let mut raw = Vec::new();
    let mut infos = Vec::new();
    for entry in ebml::children(data)?
        .into_iter()
        .filter(|c| c.id == id::TRACK_ENTRY)
    {
        let mut info = TrackInfo {
            number: 0,
            uid: 0,
            track_type: 0,
        };
        // Entries are copied unchanged, so any CRC-32 inside them stays valid.
        for child in ebml::children(entry.data)? {
            match child.id {
                id::TRACK_NUMBER => info.number = ebml::read_uint(child.data),
                id::TRACK_UID => info.uid = ebml::read_uint(child.data),
                id::TRACK_TYPE => info.track_type = ebml::read_uint(child.data),
                _ => {}
            }
        }
        raw.extend_from_slice(entry.raw);
        infos.push(info);
    }
    Ok((raw, infos))
}

/// A new subtitle TrackEntry.
pub struct NewTrack<'a> {
    pub number: u64,
    pub uid: u64,
    pub codec_id: &'a str,
    pub codec_private: &'a [u8],
    pub language: &'a str,
    pub name: Option<&'a str>,
    pub default: bool,
}

pub fn track_entry(track: &NewTrack<'_>) -> Vec<u8> {
    let mut out = Vec::new();
    ebml::put_master(&mut out, id::TRACK_ENTRY, |b| {
        ebml::put_uint(b, id::TRACK_NUMBER, track.number);
        ebml::put_uint(b, id::TRACK_UID, track.uid);
        ebml::put_uint(b, id::TRACK_TYPE, TRACK_SUBTITLE);
        ebml::put_uint(b, id::FLAG_DEFAULT, u64::from(track.default));
        ebml::put_uint(b, id::FLAG_LACING, 0);
        if let Some(name) = track.name {
            ebml::put_str(b, id::NAME, name);
        }
        ebml::put_str(b, id::LANGUAGE, legacy_language(track.language));
        ebml::put_str(b, id::LANGUAGE_BCP47, track.language);
        ebml::put_str(b, id::CODEC_ID, track.codec_id);
        ebml::put_element(b, id::CODEC_PRIVATE, track.codec_private);
    });
    out
}

/// ISO 639-2 code for the legacy Language element, read by older players.
pub fn legacy_language(tag: &str) -> &'static str {
    let primary = tag
        .split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match primary.as_str() {
        "zh" | "chi" | "zho" => "chi",
        "en" | "eng" => "eng",
        "ja" | "jpn" => "jpn",
        "ko" | "kor" => "kor",
        "fr" | "fre" | "fra" => "fre",
        "de" | "ger" | "deu" => "ger",
        "es" | "spa" => "spa",
        "it" | "ita" => "ita",
        "pt" | "por" => "por",
        "ru" | "rus" => "rus",
        "th" | "tha" => "tha",
        "vi" | "vie" => "vie",
        "ar" | "ara" => "ara",
        "id" | "ind" => "ind",
        "ms" | "may" | "msa" => "may",
        _ => "und",
    }
}

/// Existing AttachedFile elements (raw) and the FileUIDs in use.
pub fn attachments(data: &[u8]) -> io::Result<(Vec<u8>, Vec<u64>)> {
    let mut raw = Vec::new();
    let mut uids = Vec::new();
    for file in ebml::children(data)?
        .into_iter()
        .filter(|c| c.id == id::ATTACHED_FILE)
    {
        for child in ebml::children(file.data)? {
            if child.id == id::FILE_UID {
                uids.push(ebml::read_uint(child.data));
            }
        }
        raw.extend_from_slice(file.raw);
    }
    Ok((raw, uids))
}

pub fn attached_file(name: &str, media_type: &str, uid: u64, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    ebml::put_master(&mut out, id::ATTACHED_FILE, |b| {
        ebml::put_str(b, id::FILE_NAME, name);
        ebml::put_str(b, id::FILE_MEDIA_TYPE, media_type);
        ebml::put_uint(b, id::FILE_UID, uid);
        ebml::put_element(b, id::FILE_DATA, data);
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn languages() {
        assert_eq!(legacy_language("zh-Hans"), "chi");
        assert_eq!(legacy_language("ja"), "jpn");
        assert_eq!(legacy_language("tlh"), "und");
    }

    #[test]
    fn info_replaces_apps_and_reads_scale() {
        let mut data = Vec::new();
        ebml::put_uint(&mut data, id::TIMESTAMP_SCALE, 100_000);
        ebml::put_str(&mut data, id::WRITING_APP, "old");
        ebml::put_element(&mut data, id::CRC32, &[0; 4]);
        let (out, scale) = info(&data).unwrap();
        assert_eq!(scale, 100_000);
        let info = &ebml::children(&out).unwrap()[0];
        let ids: Vec<u32> = ebml::children(info.data)
            .unwrap()
            .iter()
            .map(|c| c.id)
            .collect();
        assert_eq!(ids, [id::TIMESTAMP_SCALE, id::MUXING_APP, id::WRITING_APP]);
    }
}
