//! Hand-written declarations for the subset of the HarfBuzz C API in use.
//! Signatures follow `hb-blob.h`, `hb-face.h`, `hb-set.h`, and `hb-subset.h`.
#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_uint, c_void};

#[repr(C)]
pub struct hb_blob_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct hb_face_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct hb_set_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct hb_subset_input_t {
    _private: [u8; 0],
}

pub type hb_memory_mode_t = c_uint;
pub const HB_MEMORY_MODE_READONLY: hb_memory_mode_t = 1;

pub type hb_subset_sets_t = c_uint;
pub const HB_SUBSET_SETS_NAME_ID: hb_subset_sets_t = 4;
pub const HB_SUBSET_SETS_NAME_LANG_ID: hb_subset_sets_t = 5;
pub const HB_SUBSET_SETS_LAYOUT_FEATURE_TAG: hb_subset_sets_t = 6;

pub const HB_SUBSET_FLAGS_DEFAULT: u32 = 0x0000_0000;
pub const HB_SUBSET_FLAGS_NO_HINTING: u32 = 0x0000_0001;
pub const HB_SUBSET_FLAGS_NAME_LEGACY: u32 = 0x0000_0008;

pub type hb_destroy_func_t = Option<unsafe extern "C" fn(user_data: *mut c_void)>;

unsafe extern "C" {
    pub fn hb_blob_create(
        data: *const c_char,
        length: c_uint,
        mode: hb_memory_mode_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    ) -> *mut hb_blob_t;
    pub fn hb_blob_destroy(blob: *mut hb_blob_t);
    pub fn hb_blob_get_data(blob: *mut hb_blob_t, length: *mut c_uint) -> *const c_char;

    pub fn hb_face_count(blob: *mut hb_blob_t) -> c_uint;
    pub fn hb_face_create(blob: *mut hb_blob_t, index: c_uint) -> *mut hb_face_t;
    pub fn hb_face_destroy(face: *mut hb_face_t);
    pub fn hb_face_reference_blob(face: *mut hb_face_t) -> *mut hb_blob_t;

    pub fn hb_set_add(set: *mut hb_set_t, codepoint: u32);
    pub fn hb_set_clear(set: *mut hb_set_t);
    pub fn hb_set_invert(set: *mut hb_set_t);

    pub fn hb_subset_input_create_or_fail() -> *mut hb_subset_input_t;
    pub fn hb_subset_input_destroy(input: *mut hb_subset_input_t);
    pub fn hb_subset_input_unicode_set(input: *mut hb_subset_input_t) -> *mut hb_set_t;
    pub fn hb_subset_input_set(
        input: *mut hb_subset_input_t,
        set_type: hb_subset_sets_t,
    ) -> *mut hb_set_t;
    pub fn hb_subset_input_set_flags(input: *mut hb_subset_input_t, value: c_uint);
    pub fn hb_subset_or_fail(
        source: *mut hb_face_t,
        input: *const hb_subset_input_t,
    ) -> *mut hb_face_t;
}
