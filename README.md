# mkvfontmux-rs

English | [简体中文](README-zh.md)

A Rust rewrite of [MkvFontMux](https://github.com/Shizuku-in/mkvfontmux). It finds the fonts that ASS/SSA subtitles actually use, subsets them, and muxes the subtitles together with the subset fonts into MKV files.

- Matches fonts with [ass-fonts](https://crates.io/crates/ass-fonts), aware of weight, italic and vertical `@` fonts.
- Subsets fonts with a bundled [HarfBuzz](https://github.com/harfbuzz/harfbuzz), so Python/`pyftsubset` is no longer needed.
- Gives subset fonts random names to avoid clashes with fonts installed on the viewer's system.
- Detects GBK, Big5 and Shift_JIS subtitles and caches the font index.

> A Flutter desktop GUI and a native muxer are in development. For now muxing uses `mkvmerge`.

## Requirements

- `mkvmerge` from [MKVToolNix](https://mkvtoolnix.org/)
- To build from source: Rust (stable) and a C++17 compiler

## Build

```sh
git clone --recursive https://github.com/Shizuku-in/mkvfontmux-rs
cd mkvfontmux-rs
cargo build --release
```

## Usage

```sh
mkvfontmux "[DMG&VCB-Studio] Shingeki no Kyojin Season 2 [Ma10p_1080p]" -d Fonts
```

Each `.mkv` in the directory is muxed with the `.ass`/`.ssa` files whose names start with its name. Results go to the `output` subdirectory. Run `mkvfontmux --help` for all options; they are compatible with the original tool.

See [docs/usage.md](docs/usage.md) for settings, language detection and font matching rules.

## License

[MIT](LICENSE)
