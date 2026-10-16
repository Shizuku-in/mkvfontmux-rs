import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../src/rust/api/library.dart';
import 'settings.dart';

class LibraryState {
  const LibraryState({this.library, this.progress, this.error});

  final FontLibrary? library;

  /// Non-null while scanning.
  final IndexProgress? progress;
  final String? error;

  bool get loading => library == null && error == null;
}

/// The font index, built at startup and whenever the font folders change.
class LibraryNotifier extends Notifier<LibraryState> {
  int _generation = 0;

  @override
  LibraryState build() {
    // Rebuild when the configured font folders change.
    final dirs = ref.watch(settingsProvider.select((s) => s.value?.fontDirs.join('\n')));
    if (dirs != null) {
      Future.microtask(rescan);
    }
    return const LibraryState();
  }

  Future<void> rescan() async {
    final settings = ref.read(settingsProvider).value;
    if (settings == null) return;
    final generation = ++_generation;
    state = LibraryState(library: state.library, progress: IndexProgress(scanned: 0, toScan: 0));
    try {
      final library = await FontLibrary.build(
        settings: settings,
        onProgress: (p) async {
          if (generation == _generation) {
            state = LibraryState(library: state.library, progress: p);
          }
        },
      );
      if (generation == _generation) {
        state = LibraryState(library: library);
      }
    } catch (e) {
      if (generation == _generation) {
        state = LibraryState(library: state.library, error: '$e');
      }
    }
  }
}

final libraryProvider = NotifierProvider<LibraryNotifier, LibraryState>(LibraryNotifier.new);
