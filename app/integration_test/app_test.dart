// App smoke tests, plus an end-to-end run (add a folder, preflight, mux)
// through the app state when MFM_E2E_DIR is set:
//
// flutter test integration_test -d windows --dart-define=MFM_E2E_DIR=<folder with .mkv + .ass>
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:mkvfontmux_gui/main.dart';
import 'package:mkvfontmux_gui/src/rust/api/settings.dart';
import 'package:mkvfontmux_gui/src/rust/frb_generated.dart';
import 'package:mkvfontmux_gui/state/jobs.dart';
import 'package:mkvfontmux_gui/state/library.dart';
import 'package:mkvfontmux_gui/state/settings.dart';

const _dir = String.fromEnvironment('MFM_E2E_DIR');
const _shots = String.fromEnvironment('MFM_SHOT_DIR');

/// Defaults with the built-in muxer and a dedicated output folder; never saved.
class _TestSettings extends SettingsNotifier {
  @override
  Future<SettingsDto> build() async =>
      defaultSettings().copyWith(nativeMuxer: true, outputDirName: 'out_gui');
}

Future<void> _until(bool Function() done, {Duration timeout = const Duration(minutes: 2)}) async {
  final end = DateTime.now().add(timeout);
  while (!done()) {
    if (DateTime.now().isAfter(end)) fail('timed out');
    await Future<void>.delayed(const Duration(milliseconds: 100));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());

  testWidgets('starts with the three pages and the drop hint', (tester) async {
    await tester.pumpWidget(const ProviderScope(child: MkvFontMuxApp()));
    await tester.pump(const Duration(milliseconds: 500));
    expect(find.byIcon(Icons.movie), findsOneWidget);
    expect(find.byIcon(Icons.font_download_outlined), findsOneWidget);
    expect(find.byIcon(Icons.settings_outlined), findsOneWidget);
    expect(find.byIcon(Icons.file_download_outlined), findsOneWidget);
  });

  test('bridge round-trips default settings', () {
    final settings = defaultSettings();
    expect(settings.subset, isTrue);
    expect(settings.languageRules, isNotEmpty);
    expect(systemFontDirs(), isNotEmpty);
  });

  test('adds a folder, preflights, and muxes it', () async {
    if (_dir.isEmpty) {
      markTestSkipped('set MFM_E2E_DIR');
      return;
    }
    final container = ProviderContainer(
      overrides: [settingsProvider.overrideWith(_TestSettings.new)],
    );
    addTearDown(container.dispose);
    container.listen(libraryProvider, (_, _) {});
    final jobs = container.read(jobsProvider.notifier);
    container.listen(jobsProvider, (_, _) {});

    await container.read(settingsProvider.future);
    await _until(() => container.read(libraryProvider).library != null);

    await jobs.addPaths([_dir]);
    await _until(
      () => container.read(jobsProvider).jobs.every((j) => j.plan != null && !j.analyzing),
    );
    final planned = container.read(jobsProvider).jobs;
    expect(planned, isNotEmpty);
    expect(planned.first.plan!.fonts, isNotEmpty);

    await jobs.start();
    final state = container.read(jobsProvider);
    expect(state.running, isFalse);
    for (final job in state.jobs) {
      expect(job.status, JobStatus.done, reason: job.error);
      expect(File(job.output!).existsSync(), isTrue);
      expect(job.output, contains('out_gui'));
    }
    expect(state.summary, startsWith('finished:${state.jobs.length}:0'));
  });

  testWidgets('pages render with real data', (tester) async {
    if (_dir.isEmpty) {
      markTestSkipped('set MFM_E2E_DIR');
      return;
    }
    final container = ProviderContainer(
      overrides: [settingsProvider.overrideWith(_TestSettings.new)],
    );
    addTearDown(container.dispose);
    final boundary = GlobalKey();
    await tester.binding.setSurfaceSize(const Size(1100, 760));
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: RepaintBoundary(key: boundary, child: const MkvFontMuxApp()),
      ),
    );
    Future<void> settle(bool Function() done) async {
      await tester.runAsync(() => _until(done));
      await tester.pumpAndSettle();
    }

    Future<void> shot(String name) async {
      await tester.pumpAndSettle();
      if (_shots.isEmpty) return;
      await tester.runAsync(() async {
        final render = boundary.currentContext!.findRenderObject()! as RenderRepaintBoundary;
        final image = await render.toImage();
        final png = await image.toByteData(format: ui.ImageByteFormat.png);
        await File('$_shots/$name.png').writeAsBytes(png!.buffer.asUint8List());
      });
    }

    await settle(() => container.read(libraryProvider).library != null);
    await tester.runAsync(() => container.read(jobsProvider.notifier).addPaths([_dir]));
    await settle(() => container.read(jobsProvider).jobs.every((j) => j.plan != null));
    await tester.tap(find.byType(ExpansionTile).first);
    await shot('jobs');
    expect(find.textContaining('No Such Font', findRichText: true), findsOneWidget);

    await tester.tap(find.byIcon(Icons.font_download_outlined));
    await shot('library');
    await tester.tap(find.byIcon(Icons.settings_outlined));
    await shot('settings');
  });
}
