import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../src/rust/api/settings.dart';

extension SettingsCopy on SettingsDto {
  SettingsDto copyWith({
    List<String>? fontDirs,
    bool? nativeMuxer,
    String? Function()? mkvmerge,
    bool? smartMatch,
    bool? nearestWeight,
    bool? allowSynthesis,
    bool? subset,
    bool? rename,
    bool? overwrite,
    String? outputDirName,
    bool? keepTemp,
    List<String>? ignoredFonts,
    String? fallbackLanguage,
    List<LanguageRuleDto>? languageRules,
  }) {
    return SettingsDto(
      fontDirs: fontDirs ?? this.fontDirs,
      nativeMuxer: nativeMuxer ?? this.nativeMuxer,
      mkvmerge: mkvmerge != null ? mkvmerge() : this.mkvmerge,
      smartMatch: smartMatch ?? this.smartMatch,
      nearestWeight: nearestWeight ?? this.nearestWeight,
      allowSynthesis: allowSynthesis ?? this.allowSynthesis,
      subset: subset ?? this.subset,
      rename: rename ?? this.rename,
      overwrite: overwrite ?? this.overwrite,
      outputDirName: outputDirName ?? this.outputDirName,
      keepTemp: keepTemp ?? this.keepTemp,
      ignoredFonts: ignoredFonts ?? this.ignoredFonts,
      fallbackLanguage: fallbackLanguage ?? this.fallbackLanguage,
      languageRules: languageRules ?? this.languageRules,
    );
  }
}

/// The saved settings, shared with the CLI's config.toml.
class SettingsNotifier extends AsyncNotifier<SettingsDto> {
  @override
  Future<SettingsDto> build() async {
    try {
      return await loadSettings();
    } catch (_) {
      // A broken config file should not block the app.
      return defaultSettings();
    }
  }

  Future<void> save(SettingsDto settings) async {
    await saveSettings(settings: settings);
    state = AsyncData(settings);
  }
}

final settingsProvider = AsyncNotifierProvider<SettingsNotifier, SettingsDto>(SettingsNotifier.new);
