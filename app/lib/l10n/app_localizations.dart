import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_zh.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('zh'),
  ];

  /// No description provided for @appTitle.
  ///
  /// In en, this message translates to:
  /// **'MkvFontMux'**
  String get appTitle;

  /// No description provided for @navJobs.
  ///
  /// In en, this message translates to:
  /// **'Jobs'**
  String get navJobs;

  /// No description provided for @navLibrary.
  ///
  /// In en, this message translates to:
  /// **'Fonts'**
  String get navLibrary;

  /// No description provided for @navSettings.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get navSettings;

  /// No description provided for @addFiles.
  ///
  /// In en, this message translates to:
  /// **'Add MKV files'**
  String get addFiles;

  /// No description provided for @addFolder.
  ///
  /// In en, this message translates to:
  /// **'Add folder'**
  String get addFolder;

  /// No description provided for @clear.
  ///
  /// In en, this message translates to:
  /// **'Clear'**
  String get clear;

  /// No description provided for @start.
  ///
  /// In en, this message translates to:
  /// **'Start'**
  String get start;

  /// No description provided for @cancel.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// No description provided for @dropHint.
  ///
  /// In en, this message translates to:
  /// **'Drop folders or .mkv files here'**
  String get dropHint;

  /// No description provided for @dropHintSub.
  ///
  /// In en, this message translates to:
  /// **'Subtitles next to each video (same name prefix) are picked up automatically.'**
  String get dropHintSub;

  /// No description provided for @noSubtitles.
  ///
  /// In en, this message translates to:
  /// **'No subtitles found'**
  String get noSubtitles;

  /// No description provided for @subtitlesCount.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 subtitle} other{{count} subtitles}}'**
  String subtitlesCount(int count);

  /// No description provided for @fontsSummary.
  ///
  /// In en, this message translates to:
  /// **'{found} fonts found, {missing} missing'**
  String fontsSummary(int found, int missing);

  /// No description provided for @analyzing.
  ///
  /// In en, this message translates to:
  /// **'Analyzing…'**
  String get analyzing;

  /// No description provided for @statusReady.
  ///
  /// In en, this message translates to:
  /// **'Ready'**
  String get statusReady;

  /// No description provided for @statusIssues.
  ///
  /// In en, this message translates to:
  /// **'Needs attention'**
  String get statusIssues;

  /// No description provided for @statusQueued.
  ///
  /// In en, this message translates to:
  /// **'Queued'**
  String get statusQueued;

  /// No description provided for @statusDone.
  ///
  /// In en, this message translates to:
  /// **'Done'**
  String get statusDone;

  /// No description provided for @statusFailed.
  ///
  /// In en, this message translates to:
  /// **'Failed'**
  String get statusFailed;

  /// No description provided for @statusSkipped.
  ///
  /// In en, this message translates to:
  /// **'Skipped'**
  String get statusSkipped;

  /// No description provided for @stageAnalyzing.
  ///
  /// In en, this message translates to:
  /// **'Analyzing'**
  String get stageAnalyzing;

  /// No description provided for @stageSubsetting.
  ///
  /// In en, this message translates to:
  /// **'Subsetting fonts'**
  String get stageSubsetting;

  /// No description provided for @stageRewriting.
  ///
  /// In en, this message translates to:
  /// **'Rewriting subtitles'**
  String get stageRewriting;

  /// No description provided for @stageMuxing.
  ///
  /// In en, this message translates to:
  /// **'Muxing'**
  String get stageMuxing;

  /// No description provided for @stageFinalizing.
  ///
  /// In en, this message translates to:
  /// **'Finalizing'**
  String get stageFinalizing;

  /// No description provided for @language.
  ///
  /// In en, this message translates to:
  /// **'Language'**
  String get language;

  /// No description provided for @encoding.
  ///
  /// In en, this message translates to:
  /// **'Encoding'**
  String get encoding;

  /// No description provided for @fonts.
  ///
  /// In en, this message translates to:
  /// **'Fonts'**
  String get fonts;

  /// No description provided for @missingFonts.
  ///
  /// In en, this message translates to:
  /// **'Missing fonts'**
  String get missingFonts;

  /// No description provided for @missingGlyphs.
  ///
  /// In en, this message translates to:
  /// **'Missing glyphs'**
  String get missingGlyphs;

  /// No description provided for @warnings.
  ///
  /// In en, this message translates to:
  /// **'Warnings'**
  String get warnings;

  /// No description provided for @ambiguous.
  ///
  /// In en, this message translates to:
  /// **'several candidates'**
  String get ambiguous;

  /// No description provided for @matchedAs.
  ///
  /// In en, this message translates to:
  /// **'matched as {name}'**
  String matchedAs(String name);

  /// No description provided for @styleOnly.
  ///
  /// In en, this message translates to:
  /// **'family found, style missing'**
  String get styleOnly;

  /// No description provided for @chars.
  ///
  /// In en, this message translates to:
  /// **'{count} chars'**
  String chars(int count);

  /// No description provided for @chooseFont.
  ///
  /// In en, this message translates to:
  /// **'Choose font'**
  String get chooseFont;

  /// No description provided for @changeFont.
  ///
  /// In en, this message translates to:
  /// **'Change'**
  String get changeFont;

  /// No description provided for @manual.
  ///
  /// In en, this message translates to:
  /// **'manual'**
  String get manual;

  /// No description provided for @removeOverride.
  ///
  /// In en, this message translates to:
  /// **'Use automatic match'**
  String get removeOverride;

  /// No description provided for @openFolder.
  ///
  /// In en, this message translates to:
  /// **'Open folder'**
  String get openFolder;

  /// No description provided for @remove.
  ///
  /// In en, this message translates to:
  /// **'Remove'**
  String get remove;

  /// No description provided for @log.
  ///
  /// In en, this message translates to:
  /// **'Log'**
  String get log;

  /// No description provided for @finishedSummary.
  ///
  /// In en, this message translates to:
  /// **'{succeeded} succeeded, {failed} failed'**
  String finishedSummary(int succeeded, int failed);

  /// No description provided for @cancelled.
  ///
  /// In en, this message translates to:
  /// **'Cancelled'**
  String get cancelled;

  /// No description provided for @libraryNotReady.
  ///
  /// In en, this message translates to:
  /// **'The font library is still loading.'**
  String get libraryNotReady;

  /// No description provided for @pickFontTitle.
  ///
  /// In en, this message translates to:
  /// **'Font for \"{name}\"'**
  String pickFontTitle(String name);

  /// No description provided for @search.
  ///
  /// In en, this message translates to:
  /// **'Search fonts'**
  String get search;

  /// No description provided for @browseFile.
  ///
  /// In en, this message translates to:
  /// **'Choose a font file…'**
  String get browseFile;

  /// No description provided for @noResults.
  ///
  /// In en, this message translates to:
  /// **'No matching fonts'**
  String get noResults;

  /// No description provided for @libraryTitle.
  ///
  /// In en, this message translates to:
  /// **'Font library'**
  String get libraryTitle;

  /// No description provided for @libraryStats.
  ///
  /// In en, this message translates to:
  /// **'{faces} faces in {files} files'**
  String libraryStats(int faces, int files);

  /// No description provided for @libraryCache.
  ///
  /// In en, this message translates to:
  /// **'{cached} from cache, {rescanned} scanned'**
  String libraryCache(int cached, int rescanned);

  /// No description provided for @libraryIssues.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =0{No unreadable files} =1{1 unreadable file} other{{count} unreadable files}}'**
  String libraryIssues(int count);

  /// No description provided for @scanning.
  ///
  /// In en, this message translates to:
  /// **'Scanning fonts {scanned}/{total}'**
  String scanning(int scanned, int total);

  /// No description provided for @rebuild.
  ///
  /// In en, this message translates to:
  /// **'Rescan'**
  String get rebuild;

  /// No description provided for @systemFonts.
  ///
  /// In en, this message translates to:
  /// **'System font folders'**
  String get systemFonts;

  /// No description provided for @settingsTitle.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get settingsTitle;

  /// No description provided for @save.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get save;

  /// No description provided for @saved.
  ///
  /// In en, this message translates to:
  /// **'Settings saved'**
  String get saved;

  /// No description provided for @sectionFonts.
  ///
  /// In en, this message translates to:
  /// **'Font folders'**
  String get sectionFonts;

  /// No description provided for @fontDirsHint.
  ///
  /// In en, this message translates to:
  /// **'Leave empty to use the system font folders.'**
  String get fontDirsHint;

  /// No description provided for @addDir.
  ///
  /// In en, this message translates to:
  /// **'Add folder'**
  String get addDir;

  /// No description provided for @sectionMatching.
  ///
  /// In en, this message translates to:
  /// **'Matching'**
  String get sectionMatching;

  /// No description provided for @smartMatch.
  ///
  /// In en, this message translates to:
  /// **'Strip encoding suffixes (_gbk, _big5 …) when a name is not found'**
  String get smartMatch;

  /// No description provided for @nearestWeight.
  ///
  /// In en, this message translates to:
  /// **'Use the nearest weight in a family'**
  String get nearestWeight;

  /// No description provided for @allowSynthesis.
  ///
  /// In en, this message translates to:
  /// **'Let regular faces stand in for synthesized bold/italic'**
  String get allowSynthesis;

  /// No description provided for @sectionOutput.
  ///
  /// In en, this message translates to:
  /// **'Output'**
  String get sectionOutput;

  /// No description provided for @subset.
  ///
  /// In en, this message translates to:
  /// **'Subset fonts'**
  String get subset;

  /// No description provided for @rename.
  ///
  /// In en, this message translates to:
  /// **'Give embedded fonts random names'**
  String get rename;

  /// No description provided for @overwrite.
  ///
  /// In en, this message translates to:
  /// **'Replace source files'**
  String get overwrite;

  /// No description provided for @outputDir.
  ///
  /// In en, this message translates to:
  /// **'Output subfolder'**
  String get outputDir;

  /// No description provided for @keepTemp.
  ///
  /// In en, this message translates to:
  /// **'Keep temporary files'**
  String get keepTemp;

  /// No description provided for @sectionMuxer.
  ///
  /// In en, this message translates to:
  /// **'Muxer'**
  String get sectionMuxer;

  /// No description provided for @muxerNative.
  ///
  /// In en, this message translates to:
  /// **'Built-in'**
  String get muxerNative;

  /// No description provided for @muxerMkvmerge.
  ///
  /// In en, this message translates to:
  /// **'mkvmerge'**
  String get muxerMkvmerge;

  /// No description provided for @mkvmergePath.
  ///
  /// In en, this message translates to:
  /// **'mkvmerge path'**
  String get mkvmergePath;

  /// No description provided for @mkvmergeAuto.
  ///
  /// In en, this message translates to:
  /// **'Detected: {path}'**
  String mkvmergeAuto(String path);

  /// No description provided for @mkvmergeMissing.
  ///
  /// In en, this message translates to:
  /// **'mkvmerge not found'**
  String get mkvmergeMissing;

  /// No description provided for @browse.
  ///
  /// In en, this message translates to:
  /// **'Browse…'**
  String get browse;

  /// No description provided for @sectionLanguages.
  ///
  /// In en, this message translates to:
  /// **'Subtitle languages'**
  String get sectionLanguages;

  /// No description provided for @fallbackLanguage.
  ///
  /// In en, this message translates to:
  /// **'Default language'**
  String get fallbackLanguage;

  /// No description provided for @languageRulesHint.
  ///
  /// In en, this message translates to:
  /// **'Keywords in subtitle file names, comma separated. The first matching rule wins.'**
  String get languageRulesHint;

  /// No description provided for @addRule.
  ///
  /// In en, this message translates to:
  /// **'Add rule'**
  String get addRule;

  /// No description provided for @ignoredFonts.
  ///
  /// In en, this message translates to:
  /// **'Never embed (comma separated)'**
  String get ignoredFonts;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'zh'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'zh':
      return AppLocalizationsZh();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
