//! Native Matroska remuxer: copies an existing file, appends ASS/SSA subtitle
//! tracks interleaved with the existing clusters, adds attachments, and
//! regenerates SeekHead and Cues. Block data of existing tracks is copied
//! byte for byte.

mod ass;
mod cluster;
mod ebml;
mod meta;
mod scan;

use crate::{MuxError, MuxJob, MuxObserver, MuxOutcome, Muxer};
use cluster::{CueEntry, CueRules, MAX_RELATIVE, SubBlock};
use ebml::id;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
};

/// The built-in muxer; needs no external tools.
#[derive(Debug, Default, Clone, Copy)]
pub struct NativeMuxer;

impl Muxer for NativeMuxer {
    fn name(&self) -> &'static str {
        "native"
    }

    fn mux(&self, job: &MuxJob, observer: &mut dyn MuxObserver) -> Result<MuxOutcome, MuxError> {
        if same_file(&job.input, &job.output) {
            return Err(MuxError::Failed {
                backend: "native",
                message: "output must differ from input".into(),
            });
        }
        let result = remux(job, observer);
        if result.is_err() {
            let _ = std::fs::remove_file(&job.output);
        }
        result
    }
}

fn same_file(a: &std::path::Path, b: &std::path::Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn failed(e: io::Error) -> MuxError {
    if e.kind() == io::ErrorKind::InvalidData {
        MuxError::Failed {
            backend: "native",
            message: e.to_string(),
        }
    } else {
        MuxError::Io(e)
    }
}

fn read_at(file: &mut File, offset: u64, len: u64) -> io::Result<Vec<u8>> {
    file.seek(SeekFrom::Start(offset))?;
    let mut buf = vec![0u8; usize::try_from(len).map_err(|_| ebml::invalid("element too large"))?];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

/// Data of a raw top-level element (header stripped).
fn element_data(raw: &[u8]) -> io::Result<&[u8]> {
    let mut cursor = raw;
    let header = ebml::read_header(&mut cursor)?;
    Ok(&raw[header.len..])
}

fn remux(job: &MuxJob, observer: &mut dyn MuxObserver) -> Result<MuxOutcome, MuxError> {
    let mut input = File::open(&job.input)?;
    let layout = scan::scan(&mut input).map_err(failed)?;
    let mut outcome = MuxOutcome::default();
    if layout.doc_type == "webm" {
        outcome
            .warnings
            .push("WebM input written as Matroska".into());
    }

    // Read the small top-level elements; drop the ones that are regenerated.
    let mut info = None;
    let mut tracks = None;
    let mut attachments = None;
    let mut others = Vec::new();
    for element in &layout.elements {
        match element.id {
            id::SEEK_HEAD | id::CUES | id::VOID | id::CRC32 => {}
            id::INFO if info.is_none() => {
                info = Some(read_at(&mut input, element.offset, element.len)?)
            }
            id::TRACKS if tracks.is_none() => {
                tracks = Some(read_at(&mut input, element.offset, element.len)?)
            }
            id::ATTACHMENTS if attachments.is_none() => {
                attachments = Some(read_at(&mut input, element.offset, element.len)?)
            }
            _ => others.push((
                element.id,
                read_at(&mut input, element.offset, element.len)?,
            )),
        }
    }
    let (info, scale) = match &info {
        Some(raw) => meta::info(element_data(raw)?).map_err(failed)?,
        None => meta::info(&[]).map_err(failed)?,
    };
    let tracks = tracks.ok_or_else(|| failed(ebml::invalid("no Tracks element")))?;
    let (track_entries, existing) = meta::tracks(element_data(&tracks)?).map_err(failed)?;

    // New subtitle tracks.
    let mut used_numbers: BTreeSet<u64> = existing.iter().map(|t| t.number).collect();
    let mut used_uids: BTreeSet<u64> = existing.iter().map(|t| t.uid).collect();
    let mut new_entries = Vec::new();
    let mut blocks = Vec::new();
    let mut rules = CueRules::default();
    let has_video = existing.iter().any(|t| t.track_type == meta::TRACK_VIDEO);
    for track in &existing {
        match track.track_type {
            meta::TRACK_VIDEO => {
                rules.video.insert(track.number);
            }
            meta::TRACK_AUDIO if !has_video => {
                rules.audio.insert(track.number);
            }
            meta::TRACK_SUBTITLE => {
                rules.subtitles.insert(track.number);
            }
            _ => {}
        }
    }
    let to_units = |ns: u64| (ns + scale / 2) / scale;
    for subtitle in &job.subtitles {
        let bytes = std::fs::read(&subtitle.path)?;
        let text = String::from_utf8(bytes).map_err(|_| MuxError::Failed {
            backend: "native",
            message: format!("{} is not UTF-8", subtitle.path.display()),
        })?;
        let converted = ass::convert(&text).map_err(failed)?;
        let number = (1..)
            .find(|n| !used_numbers.contains(n))
            .expect("free track number");
        used_numbers.insert(number);
        let uid = unique_uid(&mut used_uids);
        new_entries.push(meta::track_entry(&meta::NewTrack {
            number,
            uid,
            codec_id: converted.codec_id,
            codec_private: &converted.codec_private,
            language: &subtitle.language,
            name: subtitle.name.as_deref(),
            default: subtitle.default,
        }));
        rules.subtitles.insert(number);
        blocks.extend(converted.events.into_iter().map(|e| SubBlock {
            track: number,
            time: to_units(e.start_ns),
            duration: to_units(e.duration_ns),
            data: e.data,
        }));
    }
    blocks.sort_by_key(|b| b.time);

    // Metadata elements in output order.
    let mut metadata: Vec<(u32, Vec<u8>)> = vec![(id::INFO, info)];
    let mut tracks_out = Vec::new();
    ebml::put_master(&mut tracks_out, id::TRACKS, |b| {
        b.extend_from_slice(&track_entries);
        for entry in &new_entries {
            b.extend_from_slice(entry);
        }
    });
    metadata.push((id::TRACKS, tracks_out));
    let chapters: Vec<_> = others
        .iter()
        .filter(|(i, _)| *i == id::CHAPTERS)
        .cloned()
        .collect();
    metadata.extend(chapters);
    let (mut attached, mut file_uids) = match &attachments {
        Some(raw) => {
            let (files, uids) = meta::attachments(element_data(raw)?).map_err(failed)?;
            (files, uids.into_iter().collect::<BTreeSet<_>>())
        }
        None => (Vec::new(), BTreeSet::new()),
    };
    for attachment in &job.attachments {
        let data = std::fs::read(&attachment.path)?;
        let uid = unique_uid(&mut file_uids);
        attached.extend(meta::attached_file(
            &attachment.name,
            &attachment.mime_type,
            uid,
            &data,
        ));
    }
    if !attached.is_empty() {
        let mut out = Vec::new();
        ebml::put_element(&mut out, id::ATTACHMENTS, &attached);
        metadata.push((id::ATTACHMENTS, out));
    }
    metadata.extend(others.into_iter().filter(|(i, _)| *i != id::CHAPTERS));

    // SeekHead with fixed-width positions so it can be sized before positions are known.
    let mut seek_targets: Vec<u32> = metadata
        .iter()
        .map(|(i, _)| *i)
        .filter(|i| {
            matches!(
                *i,
                id::INFO | id::TRACKS | id::CHAPTERS | id::ATTACHMENTS | id::TAGS
            )
        })
        .collect();
    seek_targets.dedup();
    seek_targets.push(id::CUES);
    let seek_len = seek_head(&seek_targets, &BTreeMap::new()).0.len() as u64;
    let mut positions = BTreeMap::new();
    let mut offset = seek_len;
    for (element_id, raw) in &metadata {
        positions.entry(*element_id).or_insert(offset);
        offset += raw.len() as u64;
    }

    let file = File::create(&job.output)?;
    let mut out = Counter::new(BufWriter::with_capacity(1 << 20, file));
    out.write_all(&meta::ebml_header(
        layout.doc_type_version,
        layout.doc_type_read_version,
    ))?;
    let mut segment_header = Vec::new();
    ebml::put_id(&mut segment_header, id::SEGMENT);
    let segment_size_at = out.position + segment_header.len() as u64;
    ebml::put_size_len(&mut segment_header, 0, 8);
    out.write_all(&segment_header)?;
    let segment_data = out.position;

    let (seek, cues_patch) = seek_head(&seek_targets, &positions);
    out.write_all(&seek)?;
    for (_, raw) in &metadata {
        out.write_all(raw)?;
    }

    // Clusters with interleaved subtitles.
    let mut cues = Vec::new();
    let mut next = 0;
    let mut done_bytes = 0u64;
    let mut last_percent = 0;
    for (i, loc) in layout.clusters.iter().enumerate() {
        if observer.cancelled() {
            return Err(MuxError::Cancelled);
        }
        let next_time = layout.clusters.get(i + 1).map_or(u64::MAX, |c| c.timestamp);
        // Events before this cluster that did not fit earlier.
        next = write_orphans(
            &mut out,
            &blocks,
            next,
            loc.timestamp,
            segment_data,
            &rules,
            &mut cues,
        )?;
        let start = next;
        while next < blocks.len()
            && blocks[next].time < next_time
            && blocks[next]
                .time
                .checked_sub(loc.timestamp)
                .is_some_and(|d| d <= MAX_RELATIVE)
        {
            next += 1;
        }
        let data = read_at(&mut input, loc.offset + loc.header_len, loc.data_len)?;
        let position = out.position - segment_data;
        let (rewritten, truncated) = cluster::rewrite(
            &data,
            loc.timestamp,
            &blocks[start..next],
            &rules,
            position,
            &mut cues,
        );
        if truncated {
            outcome.warnings.push(format!(
                "cluster at byte {} is truncated; its incomplete tail was dropped",
                loc.offset
            ));
        }
        out.write_all(&rewritten)?;
        next = write_orphans(
            &mut out,
            &blocks,
            next,
            next_time,
            segment_data,
            &rules,
            &mut cues,
        )?;

        done_bytes += loc.data_len;
        let percent = (done_bytes * 100 / layout.cluster_bytes.max(1)).min(99) as u8;
        if percent != last_percent {
            last_percent = percent;
            observer.progress(percent);
        }
    }
    write_orphans(
        &mut out,
        &blocks,
        next,
        u64::MAX,
        segment_data,
        &rules,
        &mut cues,
    )?;

    // Cues, then patch the Cues seek entry and the Segment size.
    let cues_position = out.position - segment_data;
    out.write_all(&cues_element(&cues))?;
    let segment_end = out.position;
    let mut file = out.into_inner()?;
    let mut patch = Vec::new();
    ebml::put_size_len(&mut patch, segment_end - segment_data, 8);
    file.seek(SeekFrom::Start(segment_size_at))?;
    file.write_all(&patch)?;
    file.seek(SeekFrom::Start(segment_data + cues_patch))?;
    file.write_all(&cues_position.to_be_bytes())?;
    file.sync_all()?;
    observer.progress(100);
    Ok(outcome)
}

/// Write subtitle-only clusters for events starting before `limit`.
fn write_orphans(
    out: &mut Counter<impl Write>,
    blocks: &[SubBlock],
    mut next: usize,
    limit: u64,
    segment_data: u64,
    rules: &CueRules,
    cues: &mut Vec<CueEntry>,
) -> io::Result<usize> {
    while next < blocks.len() && blocks[next].time < limit {
        let timestamp = blocks[next].time;
        let start = next;
        while next < blocks.len()
            && blocks[next].time < limit
            && blocks[next].time - timestamp <= MAX_RELATIVE
        {
            next += 1;
        }
        let position = out.position - segment_data;
        out.write_all(&cluster::subtitles_only(
            timestamp,
            &blocks[start..next],
            rules,
            position,
            cues,
        ))?;
    }
    Ok(next)
}

/// SeekHead for `targets`, and the offset (from the SeekHead start) of the
/// Cues SeekPosition payload for later patching.
fn seek_head(targets: &[u32], positions: &BTreeMap<u32, u64>) -> (Vec<u8>, u64) {
    let mut body = Vec::new();
    let mut cues_payload = 0;
    for &target in targets {
        let mut seek = Vec::new();
        let mut target_id = Vec::new();
        ebml::put_id(&mut target_id, target);
        ebml::put_element(&mut seek, id::SEEK_ID, &target_id);
        ebml::put_uint_fixed(
            &mut seek,
            id::SEEK_POSITION,
            positions.get(&target).copied().unwrap_or(0),
        );
        let seek_start = body.len();
        ebml::put_element(&mut body, id::SEEK, &seek);
        if target == id::CUES {
            // The 8-byte payload is the last thing in the Seek element.
            cues_payload = (seek_start + body[seek_start..].len() - 8) as u64;
        }
    }
    let mut out = Vec::new();
    ebml::put_id(&mut out, id::SEEK_HEAD);
    // Fixed-length size so the header length does not depend on positions.
    ebml::put_size_len(&mut out, body.len() as u64, 4);
    let header_len = out.len() as u64;
    out.extend_from_slice(&body);
    (out, header_len + cues_payload)
}

fn cues_element(entries: &[CueEntry]) -> Vec<u8> {
    let mut by_time: BTreeMap<u64, Vec<&CueEntry>> = BTreeMap::new();
    for entry in entries {
        let points = by_time.entry(entry.time).or_default();
        if !points.iter().any(|p| p.track == entry.track) {
            points.push(entry);
        }
    }
    let mut out = Vec::new();
    ebml::put_master(&mut out, id::CUES, |cues| {
        for (time, points) in &by_time {
            ebml::put_master(cues, id::CUE_POINT, |point| {
                ebml::put_uint(point, id::CUE_TIME, *time);
                for p in points {
                    ebml::put_master(point, id::CUE_TRACK_POSITIONS, |pos| {
                        ebml::put_uint(pos, id::CUE_TRACK, p.track);
                        ebml::put_uint(pos, id::CUE_CLUSTER_POSITION, p.cluster_position);
                        ebml::put_uint(pos, id::CUE_RELATIVE_POSITION, p.relative_position);
                        if let Some(duration) = p.duration {
                            ebml::put_uint(pos, id::CUE_DURATION, duration);
                        }
                    });
                }
            });
        }
    });
    out
}

fn unique_uid(used: &mut BTreeSet<u64>) -> u64 {
    loop {
        let uid = rand::random::<u64>();
        if uid != 0 && used.insert(uid) {
            return uid;
        }
    }
}

/// A writer that tracks its absolute position.
struct Counter<W: Write> {
    inner: W,
    position: u64,
}

impl<W: Write> Counter<W> {
    fn new(inner: W) -> Self {
        Self { inner, position: 0 }
    }
}

impl Counter<BufWriter<File>> {
    fn into_inner(self) -> io::Result<File> {
        self.inner.into_inner().map_err(|e| e.into_error())
    }
}

impl<W: Write> Write for Counter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.position += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
