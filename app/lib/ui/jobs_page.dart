import 'package:desktop_drop/desktop_drop.dart';
import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/app_localizations.dart';
import '../state/jobs.dart';
import '../state/library.dart';
import 'episode_card.dart';

class JobsPage extends ConsumerStatefulWidget {
  const JobsPage({super.key});

  @override
  ConsumerState<JobsPage> createState() => _JobsPageState();
}

class _JobsPageState extends ConsumerState<JobsPage> {
  bool _dragging = false;
  bool _showLog = false;

  Future<void> _addFiles() async {
    final files = await openFiles(
      acceptedTypeGroups: const [
        XTypeGroup(label: 'Matroska', extensions: ['mkv']),
      ],
    );
    await ref.read(jobsProvider.notifier).addPaths([for (final f in files) f.path]);
  }

  Future<void> _addFolder() async {
    final dir = await getDirectoryPath();
    if (dir != null) await ref.read(jobsProvider.notifier).addPaths([dir]);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final state = ref.watch(jobsProvider);
    final library = ref.watch(libraryProvider);
    final notifier = ref.read(jobsProvider.notifier);
    final scheme = Theme.of(context).colorScheme;

    final summary = state.summary?.split(':');
    final summaryText = summary == null
        ? null
        : '${summary[0] == 'cancelled' ? '${l.cancelled} · ' : ''}'
              '${l.finishedSummary(int.parse(summary[1]), int.parse(summary[2]))}';

    return DropTarget(
      onDragEntered: (_) => setState(() => _dragging = true),
      onDragExited: (_) => setState(() => _dragging = false),
      onDragDone: (detail) {
        setState(() => _dragging = false);
        notifier.addPaths([for (final f in detail.files) f.path]);
      },
      child: Column(
        children: [
          Container(
            alignment: Alignment.centerLeft,
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
            child: Wrap(
              spacing: 8,
              runSpacing: 8,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                FilledButton.tonalIcon(
                  onPressed: state.running ? null : _addFiles,
                  icon: const Icon(Icons.video_file_outlined),
                  label: Text(l.addFiles),
                ),
                FilledButton.tonalIcon(
                  onPressed: state.running ? null : _addFolder,
                  icon: const Icon(Icons.folder_open_outlined),
                  label: Text(l.addFolder),
                ),
                TextButton(
                  onPressed: state.running || state.jobs.isEmpty ? null : notifier.clear,
                  child: Text(l.clear),
                ),
                if (library.progress != null)
                  Text(
                    l.scanning(library.progress!.scanned, library.progress!.toScan),
                    style: TextStyle(color: scheme.outline),
                  ),
                if (summaryText != null) Text(summaryText),
              ],
            ),
          ),
          Expanded(
            child: Stack(
              children: [
                if (state.jobs.isEmpty)
                  _EmptyHint(l: l)
                else
                  ListView.builder(
                    padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                    itemCount: state.jobs.length,
                    itemBuilder: (context, i) => EpisodeCard(index: i),
                  ),
                if (_dragging)
                  Positioned.fill(
                    child: IgnorePointer(
                      child: Container(
                        margin: const EdgeInsets.all(8),
                        decoration: BoxDecoration(
                          color: scheme.primaryContainer.withValues(alpha: 0.6),
                          border: Border.all(color: scheme.primary, width: 2),
                          borderRadius: BorderRadius.circular(12),
                        ),
                        child: Center(
                          child: Text(l.dropHint, style: Theme.of(context).textTheme.titleMedium),
                        ),
                      ),
                    ),
                  ),
              ],
            ),
          ),
          if (_showLog) _LogPanel(lines: state.log),
          Material(
            color: scheme.surfaceContainer,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              child: Row(
                children: [
                  TextButton.icon(
                    onPressed: () => setState(() => _showLog = !_showLog),
                    icon: Icon(_showLog ? Icons.expand_more : Icons.expand_less),
                    label: Text('${l.log} (${state.log.length})'),
                  ),
                  const Spacer(),
                  if (state.running)
                    FilledButton.icon(
                      onPressed: notifier.cancel,
                      style: FilledButton.styleFrom(backgroundColor: scheme.error),
                      icon: const Icon(Icons.stop),
                      label: Text(l.cancel),
                    )
                  else
                    FilledButton.icon(
                      onPressed: state.jobs.isEmpty || library.library == null
                          ? null
                          : notifier.start,
                      icon: const Icon(Icons.play_arrow),
                      label: Text(l.start),
                    ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _EmptyHint extends StatelessWidget {
  const _EmptyHint({required this.l});

  final AppLocalizations l;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.file_download_outlined, size: 56, color: theme.colorScheme.outline),
          const SizedBox(height: 12),
          Text(l.dropHint, style: theme.textTheme.titleMedium),
          const SizedBox(height: 4),
          Text(l.dropHintSub, style: TextStyle(color: theme.colorScheme.outline)),
        ],
      ),
    );
  }
}

class _LogPanel extends StatelessWidget {
  const _LogPanel({required this.lines});

  final List<String> lines;

  @override
  Widget build(BuildContext context) {
    return Container(
      height: 180,
      width: double.infinity,
      color: Theme.of(context).colorScheme.surfaceContainerLowest,
      child: ListView.builder(
        reverse: true,
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        itemCount: lines.length,
        itemBuilder: (context, i) => SelectableText(
          lines[lines.length - 1 - i],
          style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
        ),
      ),
    );
  }
}
