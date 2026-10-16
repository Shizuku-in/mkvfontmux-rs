# 使用说明

[English](usage.md) | 简体中文

## 输入与输出

输入可以是目录（只查找顶层），也可以是单个 `.mkv` 文件。同一目录下的字幕，如果文件名以视频文件名开头且紧接着一个非字母数字字符，就会被归到该视频。例如 `Ep 01.sc.ass` 属于 `Ep 01.mkv`，而 `Ep 010.ass` 不属于。

退出码：`0` 表示成功；`1` 表示部分文件失败（使用 `-p` 时表示有字体缺失）；`2` 表示参数或运行错误。

## 设置

查找顺序：

- `mkvmerge`：`-m` > 设置文件 > `MKVMERGE_BIN` > `PATH` > 常见安装位置
- 字体目录：`-d` > 设置文件 > 系统字体目录

设置文件为 `config.toml`，位于 `%APPDATA%\mkvfontmux`（Windows）、`~/Library/Application Support/mkvfontmux`（macOS）或 `~/.config/mkvfontmux`（Linux）。所有键都可以省略：

```toml
font-dirs = ['H:\Fonts', 'H:\MoreFonts']
mkvmerge = 'D:\Program Files\MKVToolNix\mkvmerge.exe'
smart-match = true        # 找不到字体名时，去掉 _gbk/_big5 等后缀再试
nearest-weight = true     # 在家族内选择最接近的字重
allow-synthesis = true    # 允许用常规字形代替粗体/斜体
subset = true
rename = true             # 子集字体使用随机家族名
keep-temp = false
ignored-fonts = ["Default", "Arial", "sans-serif"]

[output]
mode = "subdirectory"     # 或 "overwrite"
name = "output"

[languages]
fallback = "zh-Hans"
```

原版放在可执行文件旁的 `config.ini`（`mkvmerge-bin`、`font-directory`）仍会被读取。

## 语言识别

字幕轨的语言根据字幕文件名判断。规则按顺序匹配，命中第一条即停止。ASCII 关键字需要与文件名中某个完整的词相同，其他关键字只要出现在文件名中即算命中。

| 关键字 | 语言标签 |
| --- | --- |
| `tc`、`cht`、`zht`、`zh-tw`、`zh-hk`、`big5`、`繁体`、`繁中`、`繁體` | `zh-Hant` |
| `sc`、`chs`、`zhs`、`zh-cn`、`gb`、`gbk`、`gb2312`、`简体`、`简中` | `zh-Hans` |
| `en`、`eng`、`english`、`英文`、`英字` | `en` |
| `jp`、`ja`、`jpn`、`japanese`、`日文`、`日语`、`日字` | `ja` |
| 均未命中 | `-s`，否则为 `languages.fallback` |

在设置文件中用 `[[languages.rules]]`（`code`、`keywords`）定义的规则会整体替换默认规则。

## 字体匹配

- 按字体内部名称（家族名、全名、PostScript 名）匹配，不使用文件名。字体内部没有可读名称时，改用由文件名推断出的名称。
- 同一字体安装了多份时，视为同一个字体。如果一个名称匹配到确实不同的多个字形，使用第一个并给出警告。
- 子集保留全部 OpenType 布局特性（包括竖排所需的 `vert`/`vrt2`）以及字幕中用到的空白字符。改名后的子集仍保留版权和许可证等名称记录。
