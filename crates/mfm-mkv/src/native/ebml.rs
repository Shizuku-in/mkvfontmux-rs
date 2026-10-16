//! EBML primitives: element IDs, variable-length integers, and readers/writers.

use std::io::{self, Read};

/// Matroska element IDs, with their marker bits, as written in files.
pub mod id {
    pub const EBML: u32 = 0x1A45_DFA3;
    pub const EBML_VERSION: u32 = 0x4286;
    pub const EBML_READ_VERSION: u32 = 0x42F7;
    pub const EBML_MAX_ID_LENGTH: u32 = 0x42F2;
    pub const EBML_MAX_SIZE_LENGTH: u32 = 0x42F3;
    pub const DOC_TYPE: u32 = 0x4282;
    pub const DOC_TYPE_VERSION: u32 = 0x4287;
    pub const DOC_TYPE_READ_VERSION: u32 = 0x4285;

    pub const VOID: u32 = 0xEC;
    pub const CRC32: u32 = 0xBF;

    pub const SEGMENT: u32 = 0x1853_8067;
    pub const SEEK_HEAD: u32 = 0x114D_9B74;
    pub const SEEK: u32 = 0x4DBB;
    pub const SEEK_ID: u32 = 0x53AB;
    pub const SEEK_POSITION: u32 = 0x53AC;

    pub const INFO: u32 = 0x1549_A966;
    pub const TIMESTAMP_SCALE: u32 = 0x2A_D7B1;
    pub const MUXING_APP: u32 = 0x4D80;
    pub const WRITING_APP: u32 = 0x5741;

    pub const TRACKS: u32 = 0x1654_AE6B;
    pub const TRACK_ENTRY: u32 = 0xAE;
    pub const TRACK_NUMBER: u32 = 0xD7;
    pub const TRACK_UID: u32 = 0x73C5;
    pub const TRACK_TYPE: u32 = 0x83;
    pub const FLAG_DEFAULT: u32 = 0x88;
    pub const FLAG_LACING: u32 = 0x9C;
    pub const NAME: u32 = 0x536E;
    pub const LANGUAGE: u32 = 0x22_B59C;
    pub const LANGUAGE_BCP47: u32 = 0x22_B59D;
    pub const CODEC_ID: u32 = 0x86;
    pub const CODEC_PRIVATE: u32 = 0x63A2;

    pub const CHAPTERS: u32 = 0x1043_A770;
    pub const TAGS: u32 = 0x1254_C367;

    pub const ATTACHMENTS: u32 = 0x1941_A469;
    pub const ATTACHED_FILE: u32 = 0x61A7;
    pub const FILE_NAME: u32 = 0x466E;
    pub const FILE_MEDIA_TYPE: u32 = 0x4660;
    pub const FILE_DATA: u32 = 0x465C;
    pub const FILE_UID: u32 = 0x46AE;

    pub const CLUSTER: u32 = 0x1F43_B675;
    pub const TIMESTAMP: u32 = 0xE7;
    pub const POSITION: u32 = 0xA7;
    pub const PREV_SIZE: u32 = 0xAB;
    pub const SIMPLE_BLOCK: u32 = 0xA3;
    pub const BLOCK_GROUP: u32 = 0xA0;
    pub const BLOCK: u32 = 0xA1;
    pub const BLOCK_DURATION: u32 = 0x9B;
    pub const REFERENCE_BLOCK: u32 = 0xFB;

    pub const CUES: u32 = 0x1C53_BB6B;
    pub const CUE_POINT: u32 = 0xBB;
    pub const CUE_TIME: u32 = 0xB3;
    pub const CUE_TRACK_POSITIONS: u32 = 0xB7;
    pub const CUE_TRACK: u32 = 0xF7;
    pub const CUE_CLUSTER_POSITION: u32 = 0xF1;
    pub const CUE_RELATIVE_POSITION: u32 = 0xF0;
    pub const CUE_DURATION: u32 = 0xB2;

    /// Direct children of Segment; one of these ends an unknown-sized cluster.
    pub const LEVEL1: [u32; 8] = [
        SEEK_HEAD,
        INFO,
        TRACKS,
        CHAPTERS,
        CLUSTER,
        CUES,
        ATTACHMENTS,
        TAGS,
    ];
}

pub fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

/// An element header: ID, data size (`None` = unknown), and header length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub id: u32,
    pub size: Option<u64>,
    pub len: usize,
}

/// Read an element ID (1–4 bytes, marker bits kept).
pub fn read_id(r: &mut impl Read) -> io::Result<(u32, usize)> {
    let mut first = [0u8; 1];
    r.read_exact(&mut first)?;
    let len = first[0].leading_zeros() as usize + 1;
    if len > 4 {
        return Err(invalid("invalid EBML ID"));
    }
    let mut value = u32::from(first[0]);
    for _ in 1..len {
        let mut b = [0u8; 1];
        r.read_exact(&mut b)?;
        value = (value << 8) | u32::from(b[0]);
    }
    Ok((value, len))
}

/// Read a data size; all value bits set means "unknown".
pub fn read_size(r: &mut impl Read) -> io::Result<(Option<u64>, usize)> {
    let mut first = [0u8; 1];
    r.read_exact(&mut first)?;
    let len = first[0].leading_zeros() as usize + 1;
    if len > 8 {
        return Err(invalid("invalid EBML size"));
    }
    let mut value = u64::from(first[0]) & (0xFF >> len);
    let mut all_ones = value == (0xFF >> len);
    for _ in 1..len {
        let mut b = [0u8; 1];
        r.read_exact(&mut b)?;
        all_ones &= b[0] == 0xFF;
        value = (value << 8) | u64::from(b[0]);
    }
    Ok((if all_ones { None } else { Some(value) }, len))
}

pub fn read_header(r: &mut impl Read) -> io::Result<Header> {
    let (id, id_len) = read_id(r)?;
    let (size, size_len) = read_size(r)?;
    Ok(Header {
        id,
        size,
        len: id_len + size_len,
    })
}

/// A child element inside an in-memory master element.
#[derive(Debug, Clone, Copy)]
pub struct Child<'a> {
    pub id: u32,
    /// Header and data.
    pub raw: &'a [u8],
    pub data: &'a [u8],
}

/// Like [`children`], but stops at the first malformed or truncated child
/// instead of failing. Returns the complete children and whether data was cut.
pub fn children_lenient(data: &[u8]) -> (Vec<Child<'_>>, bool) {
    let mut out = Vec::new();
    let mut rest = data;
    while !rest.is_empty() {
        match children_one(rest) {
            Some(child) => {
                rest = &rest[child.raw.len()..];
                out.push(child);
            }
            None => return (out, true),
        }
    }
    (out, false)
}

fn children_one(data: &[u8]) -> Option<Child<'_>> {
    let mut cursor = data;
    let header = read_header(&mut cursor).ok()?;
    let size = match header.size {
        Some(size) => usize::try_from(size).ok()?,
        None => data.len() - header.len,
    };
    let end = header
        .len
        .checked_add(size)
        .filter(|&end| end <= data.len())?;
    Some(Child {
        id: header.id,
        raw: &data[..end],
        data: &data[header.len..end],
    })
}

/// Iterate the children of in-memory master data. Unknown sizes extend to the end.
pub fn children(mut data: &[u8]) -> io::Result<Vec<Child<'_>>> {
    let mut out = Vec::new();
    while !data.is_empty() {
        let mut cursor = data;
        let header = read_header(&mut cursor)?;
        let size = match header.size {
            Some(size) => usize::try_from(size).map_err(|_| invalid("element too large"))?,
            None => data.len() - header.len,
        };
        let end = header
            .len
            .checked_add(size)
            .filter(|&end| end <= data.len())
            .ok_or_else(|| invalid("element extends past its parent"))?;
        out.push(Child {
            id: header.id,
            raw: &data[..end],
            data: &data[header.len..end],
        });
        data = &data[end..];
    }
    Ok(out)
}

pub fn read_uint(data: &[u8]) -> u64 {
    data.iter().take(8).fold(0, |v, &b| (v << 8) | u64::from(b))
}

/// Decode a block's track number (an EBML vint) and return it with its length.
pub fn read_vint(data: &[u8]) -> Option<(u64, usize)> {
    let first = *data.first()?;
    let len = first.leading_zeros() as usize + 1;
    if len > 8 || data.len() < len {
        return None;
    }
    let mut value = u64::from(first) & (0xFF >> len);
    for &b in &data[1..len] {
        value = (value << 8) | u64::from(b);
    }
    Some((value, len))
}

/// Track number, relative timestamp, and flags of a Block/SimpleBlock payload.
pub fn block_header(data: &[u8]) -> Option<(u64, i16, u8)> {
    let (track, len) = read_vint(data)?;
    let ts = i16::from_be_bytes([*data.get(len)?, *data.get(len + 1)?]);
    Some((track, ts, *data.get(len + 2)?))
}

// ---- writing ----

pub fn put_id(out: &mut Vec<u8>, id: u32) {
    let bytes = id.to_be_bytes();
    let skip = match id {
        0..=0xFF => 3,
        0x100..=0xFFFF => 2,
        0x1_0000..=0xFF_FFFF => 1,
        _ => 0,
    };
    out.extend_from_slice(&bytes[skip..]);
}

/// Minimal-length size vint (never the reserved all-ones pattern).
pub fn put_size(out: &mut Vec<u8>, size: u64) {
    let len = (1..=8)
        .find(|&n| size < (1u64 << (7 * n)) - 1)
        .expect("size fits in 8 bytes");
    put_size_len(out, size, len);
}

/// Size vint of a fixed length, for values patched in place later.
pub fn put_size_len(out: &mut Vec<u8>, size: u64, len: usize) {
    let marked = size | (1u64 << (7 * len));
    out.extend_from_slice(&marked.to_be_bytes()[8 - len..]);
}

pub fn put_element(out: &mut Vec<u8>, id: u32, data: &[u8]) {
    put_id(out, id);
    put_size(out, data.len() as u64);
    out.extend_from_slice(data);
}

pub fn put_uint(out: &mut Vec<u8>, id: u32, value: u64) {
    let bytes = value.to_be_bytes();
    let skip = (value.leading_zeros() as usize / 8).min(7);
    put_element(out, id, &bytes[skip..]);
}

/// Unsigned integer with a fixed 8-byte payload, for values patched in place.
pub fn put_uint_fixed(out: &mut Vec<u8>, id: u32, value: u64) {
    put_element(out, id, &value.to_be_bytes());
}

pub fn put_str(out: &mut Vec<u8>, id: u32, value: &str) {
    put_element(out, id, value.as_bytes());
}

pub fn put_master(out: &mut Vec<u8>, id: u32, build: impl FnOnce(&mut Vec<u8>)) {
    let mut body = Vec::new();
    build(&mut body);
    put_element(out, id, &body);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_round_trip() {
        for size in [0u64, 1, 126, 127, 128, 16382, 16383, 1 << 40] {
            let mut buf = Vec::new();
            put_size(&mut buf, size);
            let (read, len) = read_size(&mut buf.as_slice()).unwrap();
            assert_eq!((read, len), (Some(size), buf.len()), "{size}");
        }
        // 127 needs two bytes: 0xFF alone means "unknown".
        let mut buf = Vec::new();
        put_size(&mut buf, 127);
        assert_eq!(buf, [0x40, 0x7F]);
        assert_eq!(
            read_size(&mut [0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF].as_slice())
                .unwrap()
                .0,
            None
        );
    }

    #[test]
    fn ids_and_uints() {
        let mut buf = Vec::new();
        put_uint(&mut buf, id::TIMESTAMP_SCALE, 1_000_000);
        assert_eq!(buf, [0x2A, 0xD7, 0xB1, 0x83, 0x0F, 0x42, 0x40]);
        let child = &children(&buf).unwrap()[0];
        assert_eq!(
            (child.id, read_uint(child.data)),
            (id::TIMESTAMP_SCALE, 1_000_000)
        );
        let mut zero = Vec::new();
        put_uint(&mut zero, id::FLAG_DEFAULT, 0);
        assert_eq!(zero, [0x88, 0x81, 0x00]);
    }

    #[test]
    fn block_headers() {
        assert_eq!(
            block_header(&[0x81, 0x00, 0x10, 0x80, 0xAA]),
            Some((1, 16, 0x80))
        );
        assert_eq!(
            block_header(&[0x40, 0x81, 0xFF, 0xFE, 0x00]),
            Some((129, -2, 0))
        );
        assert_eq!(block_header(&[0x81, 0x00]), None);
    }
}
