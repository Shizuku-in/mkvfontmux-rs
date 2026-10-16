// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get appTitle => 'MkvFontMux';

  @override
  String get navJobs => '任务';

  @override
  String get navLibrary => '字体库';

  @override
  String get navSettings => '设置';

  @override
  String get addFiles => '添加 MKV 文件';

  @override
  String get addFolder => '添加文件夹';

  @override
  String get clear => '清空';

  @override
  String get start => '开始';

  @override
  String get cancel => '取消';

  @override
  String get dropHint => '将文件夹或 .mkv 文件拖到这里';

  @override
  String get dropHintSub => '视频旁边同名前缀的字幕会被自动识别。';

  @override
  String get noSubtitles => '没有找到字幕';

  @override
  String subtitlesCount(int count) {
    return '$count 条字幕';
  }

  @override
  String fontsSummary(int found, int missing) {
    return '找到 $found 个字体，缺失 $missing 个';
  }

  @override
  String get analyzing => '正在分析…';

  @override
  String get statusReady => '就绪';

  @override
  String get statusIssues => '需要处理';

  @override
  String get statusQueued => '等待中';

  @override
  String get statusDone => '完成';

  @override
  String get statusFailed => '失败';

  @override
  String get statusSkipped => '已跳过';

  @override
  String get stageAnalyzing => '分析中';

  @override
  String get stageSubsetting => '子集化字体';

  @override
  String get stageRewriting => '改写字幕';

  @override
  String get stageMuxing => '封装中';

  @override
  String get stageFinalizing => '收尾';

  @override
  String get language => '语言';

  @override
  String get encoding => '编码';

  @override
  String get fonts => '字体';

  @override
  String get missingFonts => '缺失字体';

  @override
  String get missingGlyphs => '缺字';

  @override
  String get warnings => '警告';

  @override
  String get ambiguous => '有多个候选';

  @override
  String matchedAs(String name) {
    return '按 $name 匹配';
  }

  @override
  String get styleOnly => '有该字体，但缺少对应字重/斜体';

  @override
  String chars(int count) {
    return '$count 字';
  }

  @override
  String get chooseFont => '指定字体';

  @override
  String get changeFont => '更换';

  @override
  String get manual => '手动';

  @override
  String get removeOverride => '恢复自动匹配';

  @override
  String get openFolder => '打开文件夹';

  @override
  String get remove => '移除';

  @override
  String get log => '日志';

  @override
  String finishedSummary(int succeeded, int failed) {
    return '成功 $succeeded 个，失败 $failed 个';
  }

  @override
  String get cancelled => '已取消';

  @override
  String get libraryNotReady => '字体库仍在加载。';

  @override
  String pickFontTitle(String name) {
    return '为“$name”指定字体';
  }

  @override
  String get search => '搜索字体';

  @override
  String get browseFile => '选择字体文件…';

  @override
  String get noResults => '没有匹配的字体';

  @override
  String get libraryTitle => '字体库';

  @override
  String libraryStats(int faces, int files) {
    return '$files 个文件，共 $faces 个字体';
  }

  @override
  String libraryCache(int cached, int rescanned) {
    return '缓存命中 $cached 个，重新扫描 $rescanned 个';
  }

  @override
  String libraryIssues(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count 个文件无法读取',
      zero: '没有无法读取的文件',
    );
    return '$_temp0';
  }

  @override
  String scanning(int scanned, int total) {
    return '正在扫描字体 $scanned/$total';
  }

  @override
  String get rebuild => '重新扫描';

  @override
  String get systemFonts => '系统字体目录';

  @override
  String get settingsTitle => '设置';

  @override
  String get save => '保存';

  @override
  String get saved => '设置已保存';

  @override
  String get sectionFonts => '字体目录';

  @override
  String get fontDirsHint => '留空则使用系统字体目录。';

  @override
  String get addDir => '添加目录';

  @override
  String get sectionMatching => '匹配';

  @override
  String get smartMatch => '找不到时去掉编码后缀（_gbk、_big5 等）再试';

  @override
  String get nearestWeight => '在家族内选择最接近的字重';

  @override
  String get allowSynthesis => '允许用常规字形代替合成的粗体/斜体';

  @override
  String get sectionOutput => '输出';

  @override
  String get subset => '子集化字体';

  @override
  String get rename => '内嵌字体使用随机名称';

  @override
  String get overwrite => '直接替换源文件';

  @override
  String get outputDir => '输出子目录';

  @override
  String get keepTemp => '保留临时文件';

  @override
  String get sectionMuxer => '封装方式';

  @override
  String get muxerNative => '内置';

  @override
  String get muxerMkvmerge => 'mkvmerge';

  @override
  String get mkvmergePath => 'mkvmerge 路径';

  @override
  String mkvmergeAuto(String path) {
    return '已检测到：$path';
  }

  @override
  String get mkvmergeMissing => '未找到 mkvmerge';

  @override
  String get browse => '浏览…';

  @override
  String get sectionLanguages => '字幕语言';

  @override
  String get fallbackLanguage => '默认语言';

  @override
  String get languageRulesHint => '字幕文件名中的关键字，用逗号分隔。按顺序匹配，命中第一条即停止。';

  @override
  String get addRule => '添加规则';

  @override
  String get ignoredFonts => '不内嵌的字体（逗号分隔）';
}
