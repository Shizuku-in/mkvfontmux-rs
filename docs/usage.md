# Usage details

English | [简体中文](usage-zh.md)

## Inputs and output

Inputs can be directories (only the top level is searched) or individual `.mkv` files. A subtitle belongs to a video when it is in the same directory and its name starts with the video's name followed by a non-alphanumeric character. For example, `Ep 01.sc.ass` belongs to `Ep 01.mkv`, but `Ep 010.ass` does not.

Exit codes: `0` means success; `1` means some files failed (or, with `-p`, some fonts are missing); `2` means a usage or runtime error.

## Muxers

`--muxer` (or `muxer` in the settings file) selects the backend:

- `mkvmerge` (default): runs MKVToolNix's `mkvmerge`.
- `native`: the built-in muxer. It copies existing tracks byte for byte, interleaves the new subtitle tracks into the existing clusters, adds the attachments, and rebuilds SeekHead and Cues. Comment lines stay in the subtitle header, as with mkvmerge. A truncated input is muxed up to its last complete block, with a warning.

## Settings

Lookup order:

- `mkvmerge`: `-m` > settings file > `MKVMERGE_BIN` > `PATH` > default install locations
- Font directories: `-d` > settings file > system font directories

The settings file is `config.toml` in `%APPDATA%\mkvfontmux` (Windows), `~/Library/Application Support/mkvfontmux` (macOS) or `~/.config/mkvfontmux` (Linux). Every key is optional:

```toml
muxer = "mkvmerge"        # Or "native"
font-dirs = ['H:\Fonts', 'H:\MoreFonts']
mkvmerge = 'D:\Program Files\MKVToolNix\mkvmerge.exe'
smart-match = true        # Strip _gbk/_big5/... suffixes when a name is not found
nearest-weight = true     # Pick the nearest available weight within a family
allow-synthesis = true    # Let a regular face stand in for bold/italic
subset = true
rename = true             # Give subset fonts random family names
keep-temp = false
ignored-fonts = ["Default", "Arial", "sans-serif"]

[output]
mode = "subdirectory"     # Or "overwrite"
name = "output"

[languages]
fallback = "zh-Hans"
```

The original tool's `config.ini` next to the executable (`mkvmerge-bin`, `font-directory`) is still read.

## Language detection

The language of each subtitle track is decided from its file name. Rules are tried in order, and the first match wins. ASCII keywords must equal a whole token of the file name; other keywords match as substrings.

| Keywords | Tag |
| --- | --- |
| `tc`, `cht`, `zht`, `zh-tw`, `zh-hk`, `big5`, `繁体`, `繁中`, `繁體` | `zh-Hant` |
| `sc`, `chs`, `zhs`, `zh-cn`, `gb`, `gbk`, `gb2312`, `简体`, `简中` | `zh-Hans` |
| `en`, `eng`, `english`, `英文`, `英字` | `en` |
| `jp`, `ja`, `jpn`, `japanese`, `日文`, `日语`, `日字` | `ja` |
| No match | `-s`, otherwise `languages.fallback` |

`[[languages.rules]]` entries (`code`, `keywords`) in the settings file replace the default rules entirely.

## Font matching

- Fonts are matched by their internal names (family, full name and PostScript name), not by file names. For faces without any readable name, aliases derived from the file name are used instead.
- If the same font is installed more than once, the copies count as one font. When a name matches faces that really differ, the first one is used and a warning is printed.
- Subsets keep every OpenType layout feature (including `vert`/`vrt2`, needed for vertical text) and the whitespace characters the subtitles use. Renamed subsets keep their copyright and license name records.
