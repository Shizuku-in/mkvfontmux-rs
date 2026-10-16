import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/app_localizations.dart';
import '../src/rust/api/settings.dart';
import '../state/settings.dart';

class SettingsPage extends ConsumerStatefulWidget {
  const SettingsPage({super.key});

  @override
  ConsumerState<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends ConsumerState<SettingsPage> {
  /// Unsaved edits; null until the saved settings have loaded.
  SettingsDto? _draft;
  String? _detectedMkvmerge;

  void _set(SettingsDto Function(SettingsDto) update) => setState(() => _draft = update(_draft!));

  Future<void> _detect() async {
    final found = await locateMkvmerge(explicit: _draft?.mkvmerge);
    if (mounted) setState(() => _detectedMkvmerge = found ?? '');
  }

  List<String> _split(String text) =>
      text.split(RegExp(r'[,，]')).map((s) => s.trim()).where((s) => s.isNotEmpty).toList();

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final saved = ref.watch(settingsProvider).value;
    if (saved == null) return const Center(child: CircularProgressIndicator());
    if (_draft == null) {
      _draft = saved;
      _detect();
    }
    final s = _draft!;
    final theme = Theme.of(context);

    Widget section(String title, List<Widget> children) => Card(
      margin: const EdgeInsets.only(bottom: 12),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: theme.textTheme.titleMedium),
            const SizedBox(height: 8),
            ...children,
          ],
        ),
      ),
    );

    Widget toggle(String label, bool value, SettingsDto Function(bool) update) => SwitchListTile(
      contentPadding: EdgeInsets.zero,
      dense: true,
      title: Text(label),
      value: value,
      onChanged: (v) => _set((_) => update(v)),
    );

    return Column(
      children: [
        Expanded(
          child: ListView(
            padding: const EdgeInsets.all(16),
            children: [
              section(l.sectionFonts, [
                Text(l.fontDirsHint, style: TextStyle(color: theme.colorScheme.outline)),
                for (var i = 0; i < s.fontDirs.length; i++)
                  ListTile(
                    dense: true,
                    contentPadding: EdgeInsets.zero,
                    leading: const Icon(Icons.folder_outlined),
                    title: Text(s.fontDirs[i]),
                    trailing: IconButton(
                      icon: const Icon(Icons.close),
                      onPressed: () =>
                          _set((d) => d.copyWith(fontDirs: [...d.fontDirs]..removeAt(i))),
                    ),
                  ),
                TextButton.icon(
                  onPressed: () async {
                    final dir = await getDirectoryPath();
                    if (dir != null) _set((d) => d.copyWith(fontDirs: [...d.fontDirs, dir]));
                  },
                  icon: const Icon(Icons.add),
                  label: Text(l.addDir),
                ),
              ]),
              section(l.sectionMatching, [
                toggle(l.smartMatch, s.smartMatch, (v) => s.copyWith(smartMatch: v)),
                toggle(l.nearestWeight, s.nearestWeight, (v) => s.copyWith(nearestWeight: v)),
                toggle(l.allowSynthesis, s.allowSynthesis, (v) => s.copyWith(allowSynthesis: v)),
                const SizedBox(height: 8),
                TextFormField(
                  key: ValueKey('ignored-${saved.hashCode}'),
                  initialValue: s.ignoredFonts.join(', '),
                  decoration: InputDecoration(labelText: l.ignoredFonts, isDense: true),
                  onChanged: (v) => _set((d) => d.copyWith(ignoredFonts: _split(v))),
                ),
              ]),
              section(l.sectionOutput, [
                toggle(l.subset, s.subset, (v) => s.copyWith(subset: v)),
                toggle(l.rename, s.rename && s.subset, (v) => s.copyWith(rename: v)),
                toggle(l.overwrite, s.overwrite, (v) => s.copyWith(overwrite: v)),
                if (!s.overwrite)
                  TextFormField(
                    initialValue: s.outputDirName,
                    decoration: InputDecoration(labelText: l.outputDir, isDense: true),
                    onChanged: (v) => _set((d) => d.copyWith(outputDirName: v)),
                  ),
                toggle(l.keepTemp, s.keepTemp, (v) => s.copyWith(keepTemp: v)),
              ]),
              section(l.sectionMuxer, [
                SegmentedButton<bool>(
                  segments: [
                    ButtonSegment(value: false, label: Text(l.muxerMkvmerge)),
                    ButtonSegment(value: true, label: Text(l.muxerNative)),
                  ],
                  selected: {s.nativeMuxer},
                  onSelectionChanged: (v) => _set((d) => d.copyWith(nativeMuxer: v.first)),
                ),
                if (!s.nativeMuxer) ...[
                  const SizedBox(height: 8),
                  Row(
                    children: [
                      Expanded(
                        child: TextFormField(
                          key: ValueKey('mkvmerge-${s.mkvmerge}'),
                          initialValue: s.mkvmerge ?? '',
                          decoration: InputDecoration(labelText: l.mkvmergePath, isDense: true),
                          onFieldSubmitted: (v) {
                            _set(
                              (d) => d.copyWith(mkvmerge: () => v.trim().isEmpty ? null : v.trim()),
                            );
                            _detect();
                          },
                        ),
                      ),
                      TextButton(
                        onPressed: () async {
                          final file = await openFile();
                          if (file != null) {
                            _set((d) => d.copyWith(mkvmerge: () => file.path));
                            _detect();
                          }
                        },
                        child: Text(l.browse),
                      ),
                    ],
                  ),
                  if (_detectedMkvmerge != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 4),
                      child: Text(
                        _detectedMkvmerge!.isEmpty
                            ? l.mkvmergeMissing
                            : l.mkvmergeAuto(_detectedMkvmerge!),
                        style: TextStyle(
                          color: _detectedMkvmerge!.isEmpty
                              ? theme.colorScheme.error
                              : theme.colorScheme.outline,
                        ),
                      ),
                    ),
                ],
              ]),
              section(l.sectionLanguages, [
                TextFormField(
                  initialValue: s.fallbackLanguage,
                  decoration: InputDecoration(labelText: l.fallbackLanguage, isDense: true),
                  onChanged: (v) => _set((d) => d.copyWith(fallbackLanguage: v.trim())),
                ),
                const SizedBox(height: 8),
                Text(l.languageRulesHint, style: TextStyle(color: theme.colorScheme.outline)),
                for (var i = 0; i < s.languageRules.length; i++)
                  Row(
                    key: ObjectKey(s.languageRules[i]),
                    children: [
                      SizedBox(
                        width: 110,
                        child: TextFormField(
                          initialValue: s.languageRules[i].code,
                          decoration: const InputDecoration(isDense: true),
                          onChanged: (v) => _set((d) {
                            final rules = [...d.languageRules];
                            rules[i] = LanguageRuleDto(code: v.trim(), keywords: rules[i].keywords);
                            return d.copyWith(languageRules: rules);
                          }),
                        ),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: TextFormField(
                          initialValue: s.languageRules[i].keywords.join(', '),
                          decoration: const InputDecoration(isDense: true),
                          onChanged: (v) => _set((d) {
                            final rules = [...d.languageRules];
                            rules[i] = LanguageRuleDto(code: rules[i].code, keywords: _split(v));
                            return d.copyWith(languageRules: rules);
                          }),
                        ),
                      ),
                      IconButton(
                        icon: const Icon(Icons.close),
                        onPressed: () => _set(
                          (d) => d.copyWith(languageRules: [...d.languageRules]..removeAt(i)),
                        ),
                      ),
                    ],
                  ),
                TextButton.icon(
                  onPressed: () => _set(
                    (d) => d.copyWith(
                      languageRules: [
                        ...d.languageRules,
                        const LanguageRuleDto(code: 'und', keywords: []),
                      ],
                    ),
                  ),
                  icon: const Icon(Icons.add),
                  label: Text(l.addRule),
                ),
              ]),
            ],
          ),
        ),
        Material(
          color: theme.colorScheme.surfaceContainer,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Row(
              children: [
                const Spacer(),
                FilledButton.icon(
                  onPressed: () async {
                    await ref.read(settingsProvider.notifier).save(s);
                    if (context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(l.saved)));
                    }
                  },
                  icon: const Icon(Icons.save_outlined),
                  label: Text(l.save),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
