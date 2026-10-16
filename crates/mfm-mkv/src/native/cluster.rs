//! Rewriting clusters with interleaved subtitle blocks, and collecting cue points.

use super::ebml::{self, id};
use std::collections::BTreeSet;

/// A subtitle block to insert; times are in TimestampScale units.
#[derive(Debug, Clone)]
pub struct SubBlock {
    pub track: u64,
    pub time: u64,
    pub duration: u64,
    pub data: Vec<u8>,
}

/// A cue candidate; positions follow the Matroska Cues definitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CueEntry {
    pub time: u64,
    pub track: u64,
    /// Cluster offset relative to the Segment data.
    pub cluster_position: u64,
    /// Block offset relative to the cluster data.
    pub relative_position: u64,
    pub duration: Option<u64>,
}

/// Which blocks get cue points, mirroring mkvmerge's defaults: video
/// keyframes; for files without video, the first audio block per cluster;
/// every subtitle block.
#[derive(Debug, Default)]
pub struct CueRules {
    pub video: BTreeSet<u64>,
    pub audio: BTreeSet<u64>,
    pub subtitles: BTreeSet<u64>,
}

/// The largest block offset from its cluster timestamp.
pub const MAX_RELATIVE: u64 = i16::MAX as u64;

/// Rewrite one cluster's data (without its header), inserting `inserts`
/// (all with `time - timestamp <= MAX_RELATIVE` and `time >= timestamp`).
/// Returns the complete cluster element, and whether the input cluster was
/// truncated (its incomplete tail is dropped).
pub fn rewrite(
    data: &[u8],
    timestamp: u64,
    inserts: &[SubBlock],
    rules: &CueRules,
    cluster_position: u64,
    cues: &mut Vec<CueEntry>,
) -> (Vec<u8>, bool) {
    let mut body =
        Vec::with_capacity(data.len() + inserts.iter().map(|s| s.data.len() + 32).sum::<usize>());
    ebml::put_uint(&mut body, id::TIMESTAMP, timestamp);
    let mut pending = inserts.iter().peekable();
    let mut audio_cued = BTreeSet::new();

    let (children, truncated) = ebml::children_lenient(data);
    for child in children {
        match child.id {
            // Recomputed or invalidated by the rewrite.
            id::TIMESTAMP | id::CRC32 | id::VOID | id::POSITION | id::PREV_SIZE => continue,
            id::SIMPLE_BLOCK | id::BLOCK_GROUP => {
                let Some(block) = parse_block(child.id, child.data) else {
                    body.extend_from_slice(child.raw);
                    continue;
                };
                let time = timestamp.saturating_add_signed(i64::from(block.relative));
                while let Some(sub) = pending.next_if(|s| s.time < time) {
                    put_sub(&mut body, sub, timestamp, rules, cluster_position, cues);
                }
                let cue = if rules.video.contains(&block.track) {
                    block.keyframe
                } else if rules.audio.contains(&block.track) {
                    audio_cued.insert(block.track)
                } else {
                    rules.subtitles.contains(&block.track)
                };
                if cue {
                    cues.push(CueEntry {
                        time,
                        track: block.track,
                        cluster_position,
                        relative_position: body.len() as u64,
                        duration: rules
                            .subtitles
                            .contains(&block.track)
                            .then_some(block.duration)
                            .flatten(),
                    });
                }
                body.extend_from_slice(child.raw);
            }
            _ => body.extend_from_slice(child.raw),
        }
    }
    for sub in pending {
        put_sub(&mut body, sub, timestamp, rules, cluster_position, cues);
    }
    let mut out = Vec::with_capacity(body.len() + 12);
    ebml::put_element(&mut out, id::CLUSTER, &body);
    (out, truncated)
}

/// A cluster holding only subtitle blocks, for events far from any existing cluster.
pub fn subtitles_only(
    timestamp: u64,
    inserts: &[SubBlock],
    rules: &CueRules,
    cluster_position: u64,
    cues: &mut Vec<CueEntry>,
) -> Vec<u8> {
    let mut body = Vec::new();
    ebml::put_uint(&mut body, id::TIMESTAMP, timestamp);
    for sub in inserts {
        put_sub(&mut body, sub, timestamp, rules, cluster_position, cues);
    }
    let mut out = Vec::with_capacity(body.len() + 12);
    ebml::put_element(&mut out, id::CLUSTER, &body);
    out
}

fn put_sub(
    body: &mut Vec<u8>,
    sub: &SubBlock,
    timestamp: u64,
    rules: &CueRules,
    cluster_position: u64,
    cues: &mut Vec<CueEntry>,
) {
    let relative = sub.time - timestamp;
    debug_assert!(relative <= MAX_RELATIVE);
    if rules.subtitles.contains(&sub.track) {
        cues.push(CueEntry {
            time: sub.time,
            track: sub.track,
            cluster_position,
            relative_position: body.len() as u64,
            duration: Some(sub.duration),
        });
    }
    let mut block = Vec::with_capacity(sub.data.len() + 12);
    ebml::put_size(&mut block, sub.track); // track number as a vint
    block.extend_from_slice(&(relative as i16).to_be_bytes());
    block.push(0); // no lacing, not invisible
    block.extend_from_slice(&sub.data);
    ebml::put_master(body, id::BLOCK_GROUP, |g| {
        ebml::put_element(g, id::BLOCK, &block);
        ebml::put_uint(g, id::BLOCK_DURATION, sub.duration);
    });
}

struct BlockInfo {
    track: u64,
    relative: i16,
    keyframe: bool,
    duration: Option<u64>,
}

fn parse_block(element: u32, data: &[u8]) -> Option<BlockInfo> {
    if element == id::SIMPLE_BLOCK {
        let (track, relative, flags) = ebml::block_header(data)?;
        return Some(BlockInfo {
            track,
            relative,
            keyframe: flags & 0x80 != 0,
            duration: None,
        });
    }
    let mut info = None;
    let mut referenced = false;
    let mut duration = None;
    for child in ebml::children(data).ok()? {
        match child.id {
            id::BLOCK => info = ebml::block_header(child.data),
            id::REFERENCE_BLOCK => referenced = true,
            id::BLOCK_DURATION => duration = Some(ebml::read_uint(child.data)),
            _ => {}
        }
    }
    let (track, relative, _) = info?;
    Some(BlockInfo {
        track,
        relative,
        keyframe: !referenced,
        duration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_block(track: u8, relative: i16, keyframe: bool) -> Vec<u8> {
        let mut data = vec![0x80 | track];
        data.extend_from_slice(&relative.to_be_bytes());
        data.push(if keyframe { 0x80 } else { 0 });
        data.extend_from_slice(b"frame");
        let mut out = Vec::new();
        ebml::put_element(&mut out, id::SIMPLE_BLOCK, &data);
        out
    }

    #[test]
    fn inserts_in_time_order_and_records_cues() {
        let mut data = Vec::new();
        ebml::put_element(&mut data, id::CRC32, &[1, 2, 3, 4]);
        ebml::put_uint(&mut data, id::TIMESTAMP, 1000);
        data.extend(simple_block(1, 0, true));
        data.extend(simple_block(1, 40, false));
        data.extend(simple_block(1, 80, false));
        let sub = SubBlock {
            track: 3,
            time: 1050,
            duration: 2000,
            data: b"0,0,Default,,0,0,0,,Hi".to_vec(),
        };
        let rules = CueRules {
            video: [1].into(),
            subtitles: [3].into(),
            ..Default::default()
        };
        let mut cues = Vec::new();
        let (out, truncated) = rewrite(&data, 1000, &[sub], &rules, 500, &mut cues);
        assert!(!truncated);

        let cluster = &ebml::children(&out).unwrap()[0];
        let kids = ebml::children(cluster.data).unwrap();
        let ids: Vec<u32> = kids.iter().map(|c| c.id).collect();
        assert_eq!(
            ids,
            [
                id::TIMESTAMP,
                id::SIMPLE_BLOCK,
                id::SIMPLE_BLOCK,
                id::BLOCK_GROUP,
                id::SIMPLE_BLOCK
            ]
        );

        assert_eq!(cues.len(), 2);
        assert_eq!((cues[0].time, cues[0].track), (1000, 1));
        assert_eq!(
            (cues[1].time, cues[1].track, cues[1].duration),
            (1050, 3, Some(2000))
        );
        // Relative positions point at the elements inside the cluster data.
        let offset = cues[1].relative_position as usize;
        assert_eq!(cluster.data[offset], 0xA0);
        let group = &ebml::children(&cluster.data[offset..]).unwrap()[0];
        let block = &ebml::children(group.data).unwrap()[0];
        assert_eq!(ebml::block_header(block.data), Some((3, 50, 0)));
    }
}
