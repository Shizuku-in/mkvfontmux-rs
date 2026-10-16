use crate::rename::{FaceStyle, RenameError, rename_font};
use read_fonts::{FontRef, types::Tag};
use std::{collections::BTreeSet, path::PathBuf};

/// One face to subset and rename.
#[derive(Debug, Clone)]
pub struct SubsetRequest {
    pub path: PathBuf,
    pub face_index: u32,
    /// Characters to keep, including whitespace.
    pub characters: BTreeSet<char>,
    /// New family name; `None` keeps the original names (renaming disabled).
    pub family: Option<String>,
    pub style: FaceStyle,
}

/// A subset font ready to attach.
#[derive(Debug, Clone)]
pub struct SubsetResult {
    pub data: Vec<u8>,
    /// `"otf"` for CFF/CFF2 outlines, otherwise `"ttf"`.
    pub extension: &'static str,
    pub mime_type: &'static str,
}

#[derive(Debug, thiserror::Error)]
pub enum SubsetError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot subset {path}#{face_index}: {source}")]
    Subset {
        path: PathBuf,
        face_index: u32,
        source: mfm_harfbuzz::SubsetError,
    },
    #[error("cannot rename subset of {path}#{face_index}: {source}")]
    Rename {
        path: PathBuf,
        face_index: u32,
        source: RenameError,
    },
}

/// Subset `request.face_index` of the file to `request.characters` and rename it.
pub fn subset_face(request: &SubsetRequest) -> Result<SubsetResult, SubsetError> {
    let source = std::fs::read(&request.path).map_err(|source| SubsetError::Io {
        path: request.path.clone(),
        source,
    })?;
    let data = mfm_harfbuzz::subset(
        &source,
        request.face_index,
        request.characters.iter().copied(),
        &mfm_harfbuzz::SubsetOptions::default(),
    )
    .map_err(|source| SubsetError::Subset {
        path: request.path.clone(),
        face_index: request.face_index,
        source,
    })?;
    let data = match &request.family {
        Some(family) => {
            rename_font(&data, family, request.style).map_err(|source| SubsetError::Rename {
                path: request.path.clone(),
                face_index: request.face_index,
                source,
            })?
        }
        None => data,
    };
    let (extension, mime_type) = font_kind(&data);
    Ok(SubsetResult {
        data,
        extension,
        mime_type,
    })
}

/// File extension and Matroska attachment MIME type for a standalone sfnt.
///
/// Uses the legacy MIME types that older demuxers and VSFilter recognize.
pub fn font_kind(data: &[u8]) -> (&'static str, &'static str) {
    let cff = FontRef::new(data).is_ok_and(|f| {
        f.table_data(Tag::new(b"CFF ")).is_some() || f.table_data(Tag::new(b"CFF2")).is_some()
    });
    if cff {
        ("otf", "application/vnd.ms-opentype")
    } else {
        ("ttf", "application/x-truetype-font")
    }
}
