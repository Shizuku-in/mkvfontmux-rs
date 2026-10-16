//! End-to-end test of the native muxer on a synthetic Matroska file.

use mfm_mkv::{Attachment, MuxJob, Muxer, NativeMuxer, NoObserver, SubtitleTrack};

// ---- minimal EBML writing/reading for the test ----

fn id_bytes(id: u32) -> Vec<u8> {
    let b = id.to_be_bytes();
    let skip = b.iter().take_while(|&&x| x == 0).count();
    b[skip..].to_vec()
}

fn size_bytes(size: u64) -> Vec<u8> {
    let len = (1..=8).find(|&n| size < (1u64 << (7 * n)) - 1).unwrap();
    (size | (1 << (7 * len))).to_be_bytes()[8 - len..].to_vec()
}

fn el(id: u32, data: &[u8]) -> Vec<u8> {
    [id_bytes(id), size_bytes(data.len() as u64), data.to_vec()].concat()
}

fn unknown(id: u32, data: &[u8]) -> Vec<u8> {
    [
        id_bytes(id),
        vec![0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        data.to_vec(),
    ]
    .concat()
}

fn uint(id: u32, v: u64) -> Vec<u8> {
    let b = v.to_be_bytes();
    let skip = (v.leading_zeros() as usize / 8).min(7);
    el(id, &b[skip..])
}

fn simple_block(track: u8, rel: i16, key: bool) -> Vec<u8> {
    let mut d = vec![0x80 | track];
    d.extend(rel.to_be_bytes());
    d.push(if key { 0x80 } else { 0 });
    d.extend(b"video-frame");
    el(0xA3, &d)
}

struct El {
    id: u32,
    start: usize,
    data: std::ops::Range<usize>,
}

fn vint(b: &[u8], i: usize, marker: bool) -> (u64, usize) {
    let n = b[i].leading_zeros() as usize + 1;
    let mut v = if marker {
        u64::from(b[i])
    } else {
        u64::from(b[i]) & (0xFF >> n)
    };
    for k in 1..n {
        v = (v << 8) | u64::from(b[i + k]);
    }
    (v, n)
}

fn parse(b: &[u8], range: std::ops::Range<usize>) -> Vec<El> {
    let mut out = Vec::new();
    let mut i = range.start;
    while i < range.end {
        let (id, a) = vint(b, i, true);
        let (size, c) = vint(b, i + a, false);
        let data = i + a + c..i + a + c + size as usize;
        i = data.end;
        out.push(El {
            id: id as u32,
            start: data.start - a - c,
            data,
        });
    }
    out
}

fn find(b: &[u8], range: std::ops::Range<usize>, id: u32) -> Vec<El> {
    parse(b, range).into_iter().filter(|e| e.id == id).collect()
}

fn read_uint(b: &[u8], e: &El) -> u64 {
    b[e.data.clone()]
        .iter()
        .fold(0, |v, &x| (v << 8) | u64::from(x))
}

/// Write a synthetic input (CRC-32 elements, stale SeekHead/Cues, an unknown-sized
/// Segment and Cluster), a subtitle, and a font into `dir`.
fn fixture(dir: &std::path::Path) -> MuxJob {
    let input = dir.join("in.mkv");
    let crc = el(0xBF, &[0, 0, 0, 0]);

    let ebml = el(
        0x1A45DFA3,
        &[el(0x4282, b"matroska"), uint(0x4287, 2), uint(0x4285, 2)].concat(),
    );
    let info = el(
        0x1549A966,
        &[crc.clone(), uint(0x2AD7B1, 1_000_000), el(0x4D80, b"old")].concat(),
    );
    let video = el(
        0xAE,
        &[
            uint(0xD7, 1),
            uint(0x73C5, 42),
            uint(0x83, 1),
            el(0x86, b"V_TEST"),
        ]
        .concat(),
    );
    let tracks = el(0x1654AE6B, &[crc.clone(), video].concat());
    let stale_seek = el(
        0x114D9B74,
        &el(
            0x4DBB,
            &[el(0x53AB, &[0x1C, 0x53, 0xBB, 0x6B]), uint(0x53AC, 999)].concat(),
        ),
    );
    let cluster1 = el(
        0x1F43B675,
        &[
            crc.clone(),
            uint(0xE7, 0),
            simple_block(1, 0, true),
            simple_block(1, 40, false),
            simple_block(1, 80, false),
        ]
        .concat(),
    );
    // Unknown-sized cluster, terminated by the following Cues element.
    let cluster2 = unknown(
        0x1F43B675,
        &[
            uint(0xE7, 1000),
            simple_block(1, 0, true),
            simple_block(1, 40, false),
        ]
        .concat(),
    );
    let stale_cues = el(0x1C53BB6B, &el(0xBB, &uint(0xB3, 0)));
    let segment = unknown(
        0x18538067,
        &[stale_seek, info, tracks, cluster1, cluster2, stale_cues].concat(),
    );
    std::fs::write(&input, [ebml, segment].concat()).unwrap();

    let ass = dir.join("a.ass");
    std::fs::write(
        &ass,
        "\u{feff}[Script Info]\r\nScriptType: v4.00+\r\n\r\n[Events]\r\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\r\n\
         Dialogue: 0,0:00:00.05,0:00:01.00,Default,,0,0,0,,first\r\n\
         Dialogue: 0,0:00:01.02,0:00:02.00,Default,,0,0,0,,second\r\n\
         Dialogue: 0,0:01:40.00,0:01:41.00,Default,,0,0,0,,after the video\r\n",
    )
    .unwrap();
    let font = dir.join("f.ttf");
    std::fs::write(&font, b"font bytes").unwrap();

    let output = dir.join("out.mkv");
    MuxJob {
        input,
        output: output.clone(),
        subtitles: vec![SubtitleTrack {
            path: ass,
            language: "zh-Hans".into(),
            name: Some("简体".into()),
            default: true,
        }],
        attachments: vec![Attachment {
            path: font,
            name: "f.ttf".into(),
            mime_type: "application/x-truetype-font".into(),
        }],
    }
}

#[test]
fn remuxes_synthetic_file() {
    let dir = tempfile::tempdir().unwrap();
    let job = fixture(dir.path());
    let output = job.output.clone();
    NativeMuxer.mux(&job, &mut NoObserver).unwrap();

    let b = std::fs::read(&output).unwrap();
    let top = parse(&b, 0..b.len());
    assert_eq!(top.len(), 2, "EBML header and Segment");
    let seg = &top[1];
    assert_eq!(seg.data.end, b.len(), "Segment size covers the file");
    let seg_start = seg.data.start;
    let kids = parse(&b, seg.data.clone());
    let ids: Vec<u32> = kids.iter().map(|e| e.id).collect();
    assert_eq!(
        ids,
        [
            0x114D9B74, 0x1549A966, 0x1654AE6B, 0x1941A469, 0x1F43B675, 0x1F43B675, 0x1F43B675,
            0x1C53BB6B
        ],
        "SeekHead, Info, Tracks, Attachments, 3 clusters (one subtitle-only), Cues"
    );

    // SeekHead entries point at their elements.
    for seek in find(&b, kids[0].data.clone(), 0x4DBB) {
        let fields = parse(&b, seek.data.clone());
        let target = b[fields[0].data.clone()]
            .iter()
            .fold(0u32, |v, &x| (v << 8) | u32::from(x));
        let pos = read_uint(&b, &fields[1]) as usize;
        let element = kids
            .iter()
            .find(|e| e.start == seg_start + pos)
            .expect("seek target exists");
        assert_eq!(element.id, target);
    }

    // Tracks: the original entry plus the subtitle track.
    let entries = find(&b, kids[2].data.clone(), 0xAE);
    assert_eq!(entries.len(), 2);
    let sub = parse(&b, entries[1].data.clone());
    let text = |id: u32| {
        sub.iter()
            .find(|e| e.id == id)
            .map(|e| String::from_utf8_lossy(&b[e.data.clone()]).into_owned())
    };
    assert_eq!(text(0x86).as_deref(), Some("S_TEXT/ASS"));
    assert_eq!(text(0x22B59D).as_deref(), Some("zh-Hans"));
    assert_eq!(text(0x22B59C).as_deref(), Some("chi"));
    assert_eq!(text(0x536E).as_deref(), Some("简体"));
    assert!(text(0x63A2).unwrap().ends_with("Effect, Text\r\n"));

    // Clusters: CRC-32 dropped, subtitles interleaved by time.
    let clusters: Vec<&El> = kids.iter().filter(|e| e.id == 0x1F43B675).collect();
    let summary: Vec<Vec<(u32, i64)>> = clusters
        .iter()
        .map(|c| {
            parse(&b, c.data.clone())
                .iter()
                .map(|e| match e.id {
                    0xE7 => (e.id, read_uint(&b, e) as i64),
                    0xA3 => (
                        e.id,
                        i64::from(i16::from_be_bytes([
                            b[e.data.start + 1],
                            b[e.data.start + 2],
                        ])),
                    ),
                    0xA0 => {
                        let block = &find(&b, e.data.clone(), 0xA1)[0];
                        (
                            e.id,
                            i64::from(i16::from_be_bytes([
                                b[block.data.start + 1],
                                b[block.data.start + 2],
                            ])),
                        )
                    }
                    other => (other, -1),
                })
                .collect()
        })
        .collect();
    assert_eq!(
        summary[0],
        [(0xE7, 0), (0xA3, 0), (0xA3, 40), (0xA0, 50), (0xA3, 80)]
    );
    assert_eq!(
        summary[1],
        [(0xE7, 1000), (0xA3, 0), (0xA0, 20), (0xA3, 40)]
    );
    assert_eq!(summary[2], [(0xE7, 100_000), (0xA0, 0)]);

    // Cues: video keyframes and every subtitle block, all pointing at clusters.
    let points = find(&b, kids[7].data.clone(), 0xBB);
    let mut cued = Vec::new();
    for point in &points {
        let fields = parse(&b, point.data.clone());
        let time = read_uint(&b, &fields[0]);
        for pos in fields.iter().filter(|f| f.id == 0xB7) {
            let f = parse(&b, pos.data.clone());
            let cluster = seg_start + read_uint(&b, &f[1]) as usize;
            assert!(clusters.iter().any(|c| c.start == cluster));
            cued.push((time, read_uint(&b, &f[0])));
        }
    }
    cued.sort();
    assert_eq!(cued, [(0, 1), (50, 2), (1000, 1), (1020, 2), (100_000, 2)]);
}

/// Corrupt bytes throughout the input: muxing may fail, but must not panic,
/// and a failed mux leaves no output behind.
#[test]
fn survives_corrupted_input() {
    let dir = tempfile::tempdir().unwrap();
    let job = fixture(dir.path());
    let original = std::fs::read(&job.input).unwrap();
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let (mut ok, mut failed) = (0, 0);
    for round in 0..500 {
        let mut data = original.clone();
        for _ in 0..1 + round % 4 {
            let at = (next() as usize) % data.len();
            data[at] = next() as u8;
        }
        if round % 5 == 0 {
            data.truncate((next() as usize) % data.len());
        }
        std::fs::write(&job.input, &data).unwrap();
        let _ = std::fs::remove_file(&job.output);
        match NativeMuxer.mux(&job, &mut NoObserver) {
            Ok(_) => ok += 1,
            Err(_) => {
                failed += 1;
                assert!(!job.output.exists(), "partial output left after failure");
            }
        }
    }
    assert!(ok > 0 && failed > 0, "ok={ok} failed={failed}");
}
