# mkvfontmux-rs

[English](README.md) | 简体中文

[MkvFontMux](https://github.com/Shizuku-in/mkvfontmux) 的 Rust 重写版。本工具会找出 ASS/SSA 字幕实际用到的字体，将其子集化后，与字幕一起封装进 MKV 文件。

- 使用 [ass-fonts](https://crates.io/crates/ass-fonts) 匹配字体，支持字重、斜体和竖排 `@` 字体。
- 使用内置的 [HarfBuzz](https://github.com/harfbuzz/harfbuzz) 子集化字体，不再依赖 Python/`pyftsubset`。
- 子集字体使用随机名称，避免与观看者系统中已安装的字体冲突。
- 支持 GBK、Big5、Shift_JIS 等编码的字幕，并缓存字体索引。

> Flutter 桌面 GUI 和原生封装器正在开发中，目前封装使用 `mkvmerge`。

## 环境要求

- `mkvmerge`（来自 [MKVToolNix](https://mkvtoolnix.org/)）
- 从源码构建：Rust（stable）和 C++17 编译器

## 构建

```sh
git clone --recursive https://github.com/Shizuku-in/mkvfontmux-rs
cd mkvfontmux-rs
cargo build --release
```

## 使用示例

```sh
mkvfontmux "[DMG&VCB-Studio] Shingeki no Kyojin Season 2 [Ma10p_1080p]" -d Fonts
```

目录中的每个 `.mkv` 都会与文件名以其名称开头的 `.ass`/`.ssa` 字幕一起封装，结果输出到 `output` 子目录。全部参数见 `mkvfontmux --help`，与原版兼容。

设置、语言识别和字体匹配规则见 [docs/usage-zh.md](docs/usage-zh.md)。

## 许可证

[MIT](LICENSE)
