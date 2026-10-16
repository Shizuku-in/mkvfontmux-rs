import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/app_localizations.dart';
import '../src/rust/api/job.dart';
import '../src/rust/api/library.dart';
import '../state/library.dart';
import 'common.dart';

/// Let the user choose a face for font name [name]. Returns null when dismissed.
Future<FontOverride?> pickFont(BuildContext context, String name) {
  return showDialog<FontOverride>(
    context: context,
    builder: (_) => _FontPicker(name: name),
  );
}

class _FontPicker extends ConsumerStatefulWidget {
  const _FontPicker({required this.name});

  final String name;

  @override
  ConsumerState<_FontPicker> createState() => _FontPickerState();
}

class _FontPickerState extends ConsumerState<_FontPicker> {
  late final _query = TextEditingController(text: widget.name);

  @override
  void dispose() {
    _query.dispose();
    super.dispose();
  }

  Future<void> _browse() async {
    final file = await openFile(
      acceptedTypeGroups: const [
        XTypeGroup(label: 'Fonts', extensions: ['ttf', 'otf', 'ttc', 'otc']),
      ],
    );
    if (file != null && mounted) {
      Navigator.pop(context, FontOverride(name: widget.name, path: file.path, faceIndex: 0));
    }
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final library = ref.watch(libraryProvider).library;
    final List<FaceInfo> results = library?.search(query: _query.text, limit: 200) ?? [];
    final outline = Theme.of(context).colorScheme.outline;
    return AlertDialog(
      title: Text(l.pickFontTitle(widget.name)),
      content: SizedBox(
        width: 560,
        height: 420,
        child: Column(
          children: [
            TextField(
              controller: _query,
              autofocus: true,
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
              child: library == null
                  ? Center(child: Text(l.libraryNotReady))
                  : results.isEmpty
                  ? Center(child: Text(l.noResults))
                  : ListView.builder(
                      itemCount: results.length,
                      itemBuilder: (context, i) {
                        final face = results[i];
                        return ListTile(
                          dense: true,
                          title: Text('${face.family}${styleLabel(face.weight, face.italic)}'),
                          subtitle: Text(
                            '${fileName(face.path)}#${face.faceIndex} · ${face.names.take(4).join(' / ')}',
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: TextStyle(color: outline),
                          ),
                          onTap: () => Navigator.pop(
                            context,
                            FontOverride(
                              name: widget.name,
                              path: face.path,
                              faceIndex: face.faceIndex,
                            ),
                          ),
                        );
                      },
                    ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton.icon(
          onPressed: _browse,
          icon: const Icon(Icons.file_open_outlined),
          label: Text(l.browseFile),
        ),
        TextButton(onPressed: () => Navigator.pop(context), child: Text(l.cancel)),
      ],
    );
  }
}
