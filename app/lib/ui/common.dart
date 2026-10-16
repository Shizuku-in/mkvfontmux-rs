import 'dart:io';

import '../l10n/app_localizations.dart';
import '../src/rust/api/job.dart';

String fileName(String path) => path.split(RegExp(r'[\\/]')).last;

String parentDir(String path) {
  final i = path.lastIndexOf(RegExp(r'[\\/]'));
  return i <= 0 ? path : path.substring(0, i);
}

/// Reveal a file (or open a folder) in the platform file manager.
Future<void> reveal(String path) async {
  if (Platform.isWindows) {
    await Process.run('explorer', ['/select,', path]);
  } else if (Platform.isMacOS) {
    await Process.run('open', ['-R', path]);
  } else {
    await Process.run('xdg-open', [parentDir(path)]);
  }
}

String stageLabel(AppLocalizations l, RunStage stage) => switch (stage) {
  RunStage.analyzing => l.stageAnalyzing,
  RunStage.subsetting => l.stageSubsetting,
  RunStage.rewriting => l.stageRewriting,
  RunStage.muxing => l.stageMuxing,
  RunStage.finalizing => l.stageFinalizing,
};

String styleLabel(int weight, bool italic) {
  final parts = [if (weight != 400) 'w$weight', if (italic) 'italic'];
  return parts.isEmpty ? '' : ' (${parts.join(', ')})';
}
