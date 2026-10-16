import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/app_localizations.dart';
import '../src/rust/api/settings.dart';
import '../state/library.dart';
import '../state/settings.dart';
import 'common.dart';

class LibraryPage extends ConsumerStatefulWidget {
  const LibraryPage({super.key});

  @override
  ConsumerState<LibraryPage> createState() => _LibraryPageState();
}

class _LibraryPageState extends ConsumerState<LibraryPage> {
  final _query = TextEditingController();

  @override
  void dispose() {
    _query.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final state = ref.watch(libraryProvider);
    final settings = ref.watch(settingsProvider).value;
    final library = state.library;
    final stats = library?.stats;
    final dirs = settings == null || settings.fontDirs.isEmpty
        ? systemFontDirs()
        : settings.fontDirs;
    final faces = library?.search(query: _query.text, limit: 300) ?? [];

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(l.libraryTitle, style: theme.textTheme.titleLarge),
              const Spacer(),
              FilledButton.tonalIcon(
                onPressed: state.progress != null
                    ? null
                    : ref.read(libraryProvider.notifier).rescan,
                icon: const Icon(Icons.refresh),
                label: Text(l.rebuild),
              ),
            ],
          ),
          const SizedBox(height: 8),
          if (state.progress != null) ...[
            Text(l.scanning(state.progress!.scanned, state.progress!.toScan)),
            const SizedBox(height: 4),
            LinearProgressIndicator(
              value: state.progress!.toScan == 0
                  ? null
                  : state.progress!.scanned / state.progress!.toScan,
            ),
          ],
          if (state.error != null)
            Text(state.error!, style: TextStyle(color: theme.colorScheme.error)),
          if (stats != null) ...[
            Text(
              '${l.libraryStats(stats.faces, stats.files)} · ${l.libraryCache(stats.cached, stats.rescanned)}',
            ),
            Text(
              (settings?.fontDirs.isEmpty ?? true)
                  ? '${l.systemFonts}: ${dirs.join('; ')}'
                  : dirs.join('; '),
              style: TextStyle(color: theme.colorScheme.outline),
            ),
            if (stats.issues.isNotEmpty)
              ExpansionTile(
                tilePadding: EdgeInsets.zero,
                title: Text(l.libraryIssues(stats.issues.length)),
                children: [
                  SizedBox(
                    height: 120,
                    child: ListView(
                      children: [
                        for (final i in stats.issues)
                          SelectableText(i, style: const TextStyle(fontSize: 12)),
                      ],
                    ),
                  ),
                ],
              ),
          ],
          const SizedBox(height: 12),
          TextField(
            controller: _query,
            decoration: InputDecoration(
              prefixIcon: const Icon(Icons.search),
              hintText: l.search,
              border: const OutlineInputBorder(),
              isDense: true,
            ),
            onChanged: (_) => setState(() {}),
          ),
          const SizedBox(height: 8),
          Expanded(
            child: ListView.builder(
              itemCount: faces.length,
              itemBuilder: (context, i) {
                final f = faces[i];
                return ListTile(
                  dense: true,
                  title: Text('${f.family}${styleLabel(f.weight, f.italic)}'),
                  subtitle: Text(
                    '${f.path}${f.faceIndex > 0 ? '#${f.faceIndex}' : ''}\n${f.names.join(' / ')}',
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                  isThreeLine: true,
                  trailing: IconButton(
                    icon: const Icon(Icons.folder_outlined),
                    onPressed: () => reveal(f.path),
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
