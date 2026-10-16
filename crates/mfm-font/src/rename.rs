use read_fonts::{FontRef, TableProvider, types::NameId};
use write_fonts::{
    FontBuilder,
    tables::name::{Name, NameRecord},
};

/// Weight and slant of the face being renamed, used for the subfamily names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaceStyle {
    pub weight: u16,
    pub italic: bool,
}

impl FaceStyle {
    /// RIBBI subfamily name, as used by GDI-style family grouping.
    pub fn subfamily(self) -> &'static str {
        match (self.weight >= 600, self.italic) {
            (false, false) => "Regular",
            (true, false) => "Bold",
            (false, true) => "Italic",
            (true, true) => "Bold Italic",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RenameError {
    #[error("cannot parse font: {0}")]
    Read(#[from] read_fonts::ReadError),
    #[error("cannot write name table: {0}")]
    Write(#[from] write_fonts::BuilderError),
}

/// Name IDs carried over from the source: copyright, version, trademark,
/// manufacturer, designer, URLs, and license text. Everything else, including
/// typographic and WWS family names, is dropped so that the original family
/// name does not survive in the attachment.
const KEPT_NAME_IDS: [u16; 9] = [0, 5, 7, 8, 9, 11, 12, 13, 14];

/// Rebuild the name table of a standalone sfnt so the face belongs to `family`.
///
/// Writes Windows Unicode English records for family (1), subfamily (2),
/// unique ID (3), full name (4), and PostScript name (6). Only Windows Unicode
/// records of [`KEPT_NAME_IDS`] are kept from the source. Checksums are
/// recomputed. Unlike in-place patching, names are never truncated.
pub fn rename_font(data: &[u8], family: &str, style: FaceStyle) -> Result<Vec<u8>, RenameError> {
    let font = FontRef::new(data)?;
    let mut records = Vec::new();
    if let Ok(name) = font.name() {
        for record in name.name_record() {
            let windows_unicode =
                record.platform_id() == 3 && matches!(record.encoding_id(), 1 | 10);
            if !windows_unicode || !KEPT_NAME_IDS.contains(&record.name_id().to_u16()) {
                continue;
            }
            let Ok(string) = record.string(name.string_data()) else {
                continue;
            };
            records.push(NameRecord::new(
                3,
                record.encoding_id(),
                record.language_id(),
                record.name_id(),
                string.chars().collect::<String>().into(),
            ));
        }
    }

    let subfamily = style.subfamily();
    let full = if subfamily == "Regular" {
        family.to_owned()
    } else {
        format!("{family} {subfamily}")
    };
    let postscript = format!("{}-{}", postscript_safe(family), subfamily.replace(' ', ""));
    for (id, value) in [
        (1, family.to_owned()),
        (2, subfamily.to_owned()),
        (3, format!("{postscript};mkvfontmux")),
        (4, full),
        (6, postscript),
    ] {
        records.push(NameRecord::new(3, 1, 0x0409, NameId::new(id), value.into()));
    }
    records.sort_by_key(|r| (r.platform_id, r.encoding_id, r.language_id, r.name_id));
    records.dedup_by_key(|r| (r.platform_id, r.encoding_id, r.language_id, r.name_id));

    let table = Name::new(records);
    Ok(FontBuilder::new()
        .add_table(&table)?
        .copy_missing_tables(font)
        .build())
}

/// PostScript names allow printable ASCII except `[](){}<>/%` and space, max 63 bytes.
fn postscript_safe(name: &str) -> String {
    let safe: String = name
        .chars()
        .filter(|c| c.is_ascii_graphic() && !"[](){}<>/%".contains(*c))
        .take(40)
        .collect();
    if safe.is_empty() { "Font".into() } else { safe }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subfamily_names() {
        assert_eq!(
            FaceStyle {
                weight: 400,
                italic: false
            }
            .subfamily(),
            "Regular"
        );
        assert_eq!(
            FaceStyle {
                weight: 700,
                italic: true
            }
            .subfamily(),
            "Bold Italic"
        );
    }

    #[test]
    fn postscript_names_are_sanitized() {
        assert_eq!(postscript_safe("Ab c(d)"), "Abcd");
        assert_eq!(postscript_safe("字体"), "Font");
    }
}
