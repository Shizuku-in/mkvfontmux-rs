//! First pass: locate the EBML header, Segment children, and clusters
//! without reading block data.

use super::ebml::{self, Header, id, invalid};
use std::io::{self, BufReader, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct Layout {
    pub doc_type: String,
    pub doc_type_version: u64,
    pub doc_type_read_version: u64,
    /// Absolute offset of the Segment's data.
    pub segment_data: u64,
    /// Non-cluster Segment children, in file order.
    pub elements: Vec<Element>,
    pub clusters: Vec<ClusterLoc>,
    /// Bytes of cluster data, for progress reporting.
    pub cluster_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Element {
    pub id: u32,
    /// Absolute offset of the element header.
    pub offset: u64,
    /// Header plus data length.
    pub len: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct ClusterLoc {
    pub offset: u64,
    pub header_len: u64,
    pub data_len: u64,
    /// Cluster timestamp in TimestampScale units.
    pub timestamp: u64,
}

pub fn scan<R: Read + Seek>(input: R) -> io::Result<Layout> {
    let mut r = BufReader::with_capacity(64 * 1024, input);
    let file_len = r.seek(SeekFrom::End(0))?;
    r.seek(SeekFrom::Start(0))?;

    let header = ebml::read_header(&mut r)?;
    if header.id != id::EBML {
        return Err(invalid("not an EBML file"));
    }
    let ebml_size = header
        .size
        .filter(|&size| size <= 4096)
        .ok_or_else(|| invalid("EBML header of unknown or implausible size"))?;
    let mut ebml_data = vec![0u8; ebml_size as usize];
    r.read_exact(&mut ebml_data)?;
    let mut layout = Layout {
        doc_type: "matroska".into(),
        doc_type_version: 1,
        doc_type_read_version: 1,
        segment_data: 0,
        elements: Vec::new(),
        clusters: Vec::new(),
        cluster_bytes: 0,
    };
    for child in ebml::children(&ebml_data)? {
        match child.id {
            id::DOC_TYPE => {
                layout.doc_type = String::from_utf8_lossy(child.data)
                    .trim_end_matches('\0')
                    .into()
            }
            id::DOC_TYPE_VERSION => layout.doc_type_version = ebml::read_uint(child.data),
            id::DOC_TYPE_READ_VERSION => layout.doc_type_read_version = ebml::read_uint(child.data),
            _ => {}
        }
    }
    if layout.doc_type != "matroska" && layout.doc_type != "webm" {
        return Err(invalid(format!(
            "unsupported DocType {:?}",
            layout.doc_type
        )));
    }

    // Skip anything (such as Void) between the EBML header and the Segment.
    let segment = loop {
        let h = ebml::read_header(&mut r)?;
        if h.id == id::SEGMENT {
            break h;
        }
        let size = h
            .size
            .ok_or_else(|| invalid("unknown-sized element before Segment"))?;
        r.seek(SeekFrom::Current(size as i64))?;
    };
    layout.segment_data = r.stream_position()?;
    let end = segment
        .size
        .map_or(file_len, |size| (layout.segment_data + size).min(file_len));

    let mut pos = layout.segment_data;
    while pos < end {
        r.seek(SeekFrom::Start(pos))?;
        let h = match ebml::read_header(&mut r) {
            Ok(h) => h,
            // Trailing garbage or a truncated header ends the Segment.
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        };
        if h.id == id::EBML {
            // A second Segment (concatenated file); only the first is used.
            break;
        }
        let data_start = pos + h.len as u64;
        if h.id == id::CLUSTER {
            let (data_len, timestamp) = scan_cluster(&mut r, h, data_start, end)?;
            layout.clusters.push(ClusterLoc {
                offset: pos,
                header_len: h.len as u64,
                data_len,
                timestamp,
            });
            layout.cluster_bytes += data_len;
            pos = data_start + data_len;
        } else {
            let size = h
                .size
                .ok_or_else(|| invalid(format!("unknown-sized element 0x{:X}", h.id)))?;
            let len = (h.len as u64 + size).min(end - pos);
            layout.elements.push(Element {
                id: h.id,
                offset: pos,
                len,
            });
            pos += len;
        }
    }
    Ok(layout)
}

/// Find a cluster's data length (resolving unknown sizes) and its Timestamp.
fn scan_cluster<R: Read + Seek>(
    r: &mut R,
    h: Header,
    data_start: u64,
    end: u64,
) -> io::Result<(u64, u64)> {
    let known_end = h.size.map(|s| (data_start + s).min(end));
    let mut pos = data_start;
    let mut timestamp = None;
    loop {
        let limit = known_end.unwrap_or(end);
        if pos >= limit || (known_end.is_some() && timestamp.is_some()) {
            break;
        }
        r.seek(SeekFrom::Start(pos))?;
        let child = match ebml::read_header(r) {
            Ok(child) => child,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        };
        if known_end.is_none() && (id::LEVEL1.contains(&child.id) || child.id == id::EBML) {
            break;
        }
        let size = child
            .size
            .ok_or_else(|| invalid("unknown-sized cluster child"))?;
        if child.id == id::TIMESTAMP {
            let mut buf = vec![0u8; size.min(8) as usize];
            r.read_exact(&mut buf)?;
            timestamp = Some(ebml::read_uint(&buf));
        }
        pos += child.len as u64 + size;
    }
    let data_end = known_end.unwrap_or(pos.min(end));
    let timestamp =
        timestamp.ok_or_else(|| invalid(format!("cluster at {} has no Timestamp", data_start)))?;
    Ok((data_end - data_start, timestamp))
}
