//! Font handling for mkvfontmux: a cached font index on top of `ass-fonts`,
//! HarfBuzz subsetting, and name-table rewriting for obfuscated attachments.

mod cache;
mod dirs;
mod rename;
mod subset;

pub use cache::{FontCache, IndexProgress, IndexStats, build_index};
pub use dirs::system_font_dirs;
pub use rename::{FaceStyle, RenameError, rename_font};
pub use subset::{SubsetError, SubsetRequest, SubsetResult, font_kind, subset_face};
