//! Font subsetting through a statically linked HarfBuzz.
//!
//! Only the handful of HarfBuzz entry points needed for subsetting are declared,
//! so building does not require bindgen or libclang.

mod ffi;

use std::{ffi::c_uint, ptr::NonNull};

/// Why a subset could not be produced.
#[derive(Debug, thiserror::Error)]
pub enum SubsetError {
    #[error("font data is larger than 4 GiB")]
    TooLarge,
    #[error("face index {index} is out of range (font has {count} faces)")]
    FaceIndex { index: u32, count: u32 },
    #[error("HarfBuzz could not allocate the subset input")]
    Allocation,
    #[error("HarfBuzz failed to subset the font")]
    Failed,
}

/// Options mirroring the pyftsubset invocation of the original tool.
#[derive(Debug, Clone)]
pub struct SubsetOptions {
    /// Keep every OpenType layout feature (`--layout-features=*`). Required for
    /// vertical forms (`vert`/`vrt2`) used by `@` fonts.
    pub all_layout_features: bool,
    /// Keep every name record, including non-English and legacy ones, so the
    /// caller can decide what to keep when rebuilding the name table.
    pub all_names: bool,
    /// Drop hinting instructions.
    pub no_hinting: bool,
}

impl Default for SubsetOptions {
    fn default() -> Self {
        Self {
            all_layout_features: true,
            all_names: true,
            no_hinting: false,
        }
    }
}

/// Number of faces in a font file: 1 for TTF/OTF, the collection size for TTC/OTC,
/// 0 if HarfBuzz does not recognize the data.
pub fn face_count(data: &[u8]) -> Result<u32, SubsetError> {
    let blob = Blob::new(data)?;
    // SAFETY: `blob` is a valid HarfBuzz blob for the duration of the call.
    Ok(unsafe { ffi::hb_face_count(blob.0.as_ptr()) })
}

/// Subset face `face_index` of `data` to the glyphs needed for `unicodes`.
///
/// The result is always a standalone sfnt (TTF or CFF-flavored OTF), even when
/// the input is a collection.
pub fn subset(
    data: &[u8],
    face_index: u32,
    unicodes: impl IntoIterator<Item = char>,
    options: &SubsetOptions,
) -> Result<Vec<u8>, SubsetError> {
    let blob = Blob::new(data)?;
    // SAFETY: all HarfBuzz objects are created and destroyed in this function,
    // and the input blob borrows `data`, which outlives every object using it.
    unsafe {
        let count = ffi::hb_face_count(blob.0.as_ptr());
        if face_index >= count.max(1) {
            return Err(SubsetError::FaceIndex {
                index: face_index,
                count,
            });
        }
        let face = Face(
            NonNull::new(ffi::hb_face_create(blob.0.as_ptr(), face_index))
                .ok_or(SubsetError::Failed)?,
        );

        let input = Input(
            NonNull::new(ffi::hb_subset_input_create_or_fail()).ok_or(SubsetError::Allocation)?,
        );
        let input_ptr = input.0.as_ptr();

        let set = ffi::hb_subset_input_unicode_set(input_ptr);
        for c in unicodes {
            ffi::hb_set_add(set, c as u32);
        }
        if options.all_layout_features {
            let features =
                ffi::hb_subset_input_set(input_ptr, ffi::HB_SUBSET_SETS_LAYOUT_FEATURE_TAG);
            ffi::hb_set_clear(features);
            ffi::hb_set_invert(features);
        }
        let mut flags = ffi::HB_SUBSET_FLAGS_DEFAULT;
        if options.all_names {
            for which in [
                ffi::HB_SUBSET_SETS_NAME_ID,
                ffi::HB_SUBSET_SETS_NAME_LANG_ID,
            ] {
                let set = ffi::hb_subset_input_set(input_ptr, which);
                ffi::hb_set_clear(set);
                ffi::hb_set_invert(set);
            }
            flags |= ffi::HB_SUBSET_FLAGS_NAME_LEGACY;
        }
        if options.no_hinting {
            flags |= ffi::HB_SUBSET_FLAGS_NO_HINTING;
        }
        ffi::hb_subset_input_set_flags(input_ptr, flags as c_uint);

        let result = Face(
            NonNull::new(ffi::hb_subset_or_fail(face.0.as_ptr(), input_ptr))
                .ok_or(SubsetError::Failed)?,
        );
        let out_blob = ffi::hb_face_reference_blob(result.0.as_ptr());
        let mut len: c_uint = 0;
        let ptr = ffi::hb_blob_get_data(out_blob, &mut len);
        let bytes = if ptr.is_null() || len == 0 {
            None
        } else {
            Some(std::slice::from_raw_parts(ptr.cast::<u8>(), len as usize).to_vec())
        };
        ffi::hb_blob_destroy(out_blob);
        bytes.ok_or(SubsetError::Failed)
    }
}

struct Blob(NonNull<ffi::hb_blob_t>);

impl Blob {
    fn new(data: &[u8]) -> Result<Self, SubsetError> {
        let len = c_uint::try_from(data.len()).map_err(|_| SubsetError::TooLarge)?;
        // SAFETY: READONLY mode makes HarfBuzz copy before any write; callers keep
        // `data` alive for as long as the blob is used.
        let ptr = unsafe {
            ffi::hb_blob_create(
                data.as_ptr().cast(),
                len,
                ffi::HB_MEMORY_MODE_READONLY,
                std::ptr::null_mut(),
                None,
            )
        };
        NonNull::new(ptr).map(Self).ok_or(SubsetError::Allocation)
    }
}

impl Drop for Blob {
    fn drop(&mut self) {
        // SAFETY: we own one reference.
        unsafe { ffi::hb_blob_destroy(self.0.as_ptr()) }
    }
}

struct Face(NonNull<ffi::hb_face_t>);

impl Drop for Face {
    fn drop(&mut self) {
        // SAFETY: we own one reference.
        unsafe { ffi::hb_face_destroy(self.0.as_ptr()) }
    }
}

struct Input(NonNull<ffi::hb_subset_input_t>);

impl Drop for Input {
    fn drop(&mut self) {
        // SAFETY: we own one reference.
        unsafe { ffi::hb_subset_input_destroy(self.0.as_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_garbage() {
        assert_eq!(face_count(b"not a font").unwrap(), 0);
        assert!(subset(b"not a font", 0, ['a'], &SubsetOptions::default()).is_err());
    }
}
