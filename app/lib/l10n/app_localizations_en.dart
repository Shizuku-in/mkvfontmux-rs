// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'MkvFontMux';

  @override
  String get navJobs => 'Jobs';

  @override
  String get navLibrary => 'Fonts';

  @override
  String get navSettings => 'Settings';

  @override
  String get addFiles => 'Add MKV files';

  @override
  String get addFolder => 'Add folder';

  @override
  String get clear => 'Clear';

  @override
  String get start => 'Start';

  @override
  String get cancel => 'Cancel';

  @override
  String get dropHint => 'Drop folders or .mkv files here';

  @override
  String get dropHintSub =>
      'Subtitles next to each video (same name prefix) are picked up automatically.';

  @override
  String get noSubtitles => 'No subtitles found';

  @override
  String subtitlesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count subtitles',
      one: '1 subtitle',
    );
    return '$_temp0';
  }

  @override
  String fontsSummary(int found, int missing) {
    return '$found fonts found, $missing missing';
  }

  @override
  String get analyzing => 'Analyzing…';

  @override
  String get statusReady => 'Ready';

  @override
  String get statusIssues => 'Needs attention';

  @override
  String get statusQueued => 'Queued';

  @override
  String get statusDone => 'Done';

  @override
  String get statusFailed => 'Failed';

  @override
  String get statusSkipped => 'Skipped';

  @override
  String get stageAnalyzing => 'Analyzing';

  @override
  String get stageSubsetting => 'Subsetting fonts';

  @override
  String get stageRewriting => 'Rewriting subtitles';

  @override
  String get stageMuxing => 'Muxing';

  @override
  String get stageFinalizing => 'Finalizing';

  @override
  String get language => 'Language';

  @override
  String get encoding => 'Encoding';

  @override
  String get fonts => 'Fonts';

  @override
  String get missingFonts => 'Missing fonts';

  @override
  String get missingGlyphs => 'Missing glyphs';

  @override
  String get warnings => 'Warnings';

  @override
  String get ambiguous => 'several candidates';

  @override
  String matchedAs(String name) {
    return 'matched as $name';
  }

  @override
  String get styleOnly => 'family found, style missing';

  @override
  String chars(int count) {
    return '$count chars';
  }

  @override
  String get chooseFont => 'Choose font';

  @override
  String get changeFont => 'Change';

  @override
  String get manual => 'manual';

  @override
  String get removeOverride => 'Use automatic match';

  @override
  String get openFolder => 'Open folder';

  @override
  String get remove => 'Remove';

  @override
  String get log => 'Log';

  @override
  String finishedSummary(int succeeded, int failed) {
    return '$succeeded succeeded, $failed failed';
  }

  @override
  String get cancelled => 'Cancelled';

  @override
  String get libraryNotReady => 'The font library is still loading.';

  @override
  String pickFontTitle(String name) {
    return 'Font for \"$name\"';
  }

  @override
  String get search => 'Search fonts';

  @override
  String get browseFile => 'Choose a font file…';

  @override
  String get noResults => 'No matching fonts';

  @override
  String get libraryTitle => 'Font library';

  @override
  String libraryStats(int faces, int files) {
    return '$faces faces in $files files';
  }

  @override
  String libraryCache(int cached, int rescanned) {
    return '$cached from cache, $rescanned scanned';
  }

  @override
  String libraryIssues(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count unreadable files',
      one: '1 unreadable file',
      zero: 'No unreadable files',
    );
    return '$_temp0';
  }

  @override
  String scanning(int scanned, int total) {
    return 'Scanning fonts $scanned/$total';
  }

  @override
  String get rebuild => 'Rescan';

  @override
  String get systemFonts => 'System font folders';

  @override
  String get settingsTitle => 'Settings';

  @override
  String get save => 'Save';

  @override
  String get saved => 'Settings saved';

  @override
  String get sectionFonts => 'Font folders';

  @override
  String get fontDirsHint => 'Leave empty to use the system font folders.';

  @override
  String get addDir => 'Add folder';

  @override
  String get sectionMatching => 'Matching';

  @override
  String get smartMatch =>
      'Strip encoding suffixes (_gbk, _big5 …) when a name is not found';

  @override
  String get nearestWeight => 'Use the nearest weight in a family';

  @override
  String get allowSynthesis =>
      'Let regular faces stand in for synthesized bold/italic';

  @override
  String get sectionOutput => 'Output';

  @override
  String get subset => 'Subset fonts';

  @override
  String get rename => 'Give embedded fonts random names';

  @override
  String get overwrite => 'Replace source files';

  @override
  String get outputDir => 'Output subfolder';

  @override
  String get keepTemp => 'Keep temporary files';

  @override
  String get sectionMuxer => 'Muxer';

  @override
  String get muxerNative => 'Built-in';

  @override
  String get muxerMkvmerge => 'mkvmerge';

  @override
  String get mkvmergePath => 'mkvmerge path';

  @override
  String mkvmergeAuto(String path) {
    return 'Detected: $path';
  }

  @override
  String get mkvmergeMissing => 'mkvmerge not found';

  @override
  String get browse => 'Browse…';

  @override
  String get sectionLanguages => 'Subtitle languages';

  @override
  String get fallbackLanguage => 'Default language';

  @override
  String get languageRulesHint =>
      'Keywords in subtitle file names, comma separated. The first matching rule wins.';

  @override
  String get addRule => 'Add rule';

  @override
  String get ignoredFonts => 'Never embed (comma separated)';
}
