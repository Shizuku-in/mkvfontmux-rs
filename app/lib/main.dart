import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'l10n/app_localizations.dart';
import 'src/rust/frb_generated.dart';
import 'state/jobs.dart';
import 'state/library.dart';
import 'ui/jobs_page.dart';
import 'ui/library_page.dart';
import 'ui/settings_page.dart';

Future<void> main(List<String> args) async {
  await RustLib.init();
  final container = ProviderContainer();
  // Paths given on the command line (or dropped onto the executable) become jobs.
  if (args.isNotEmpty) container.read(jobsProvider.notifier).addPaths(args);
  runApp(UncontrolledProviderScope(container: container, child: const MkvFontMuxApp()));
}

class MkvFontMuxApp extends StatelessWidget {
  const MkvFontMuxApp({super.key});

  @override
  Widget build(BuildContext context) {
    ThemeData theme(Brightness brightness) => ThemeData(
      colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF3F6FB5), brightness: brightness),
      visualDensity: VisualDensity.compact,
    );
    return MaterialApp(
      onGenerateTitle: (context) => AppLocalizations.of(context).appTitle,
      debugShowCheckedModeBanner: false,
      theme: theme(Brightness.light),
      darkTheme: theme(Brightness.dark),
      localizationsDelegates: const [
        AppLocalizations.delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      supportedLocales: AppLocalizations.supportedLocales,
      home: const HomePage(),
    );
  }
}

class HomePage extends ConsumerStatefulWidget {
  const HomePage({super.key});

  @override
  ConsumerState<HomePage> createState() => _HomePageState();
}

class _HomePageState extends ConsumerState<HomePage> {
  int _page = 0;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    // Keep the library and job state alive regardless of the visible page.
    ref.watch(libraryProvider);
    final running = ref.watch(jobsProvider.select((s) => s.running));
    return Scaffold(
      body: Row(
        children: [
          NavigationRail(
            selectedIndex: _page,
            labelType: NavigationRailLabelType.all,
            onDestinationSelected: (i) => setState(() => _page = i),
            destinations: [
              NavigationRailDestination(
                icon: Badge(isLabelVisible: running, child: const Icon(Icons.movie_outlined)),
                selectedIcon: const Icon(Icons.movie),
                label: Text(l.navJobs),
              ),
              NavigationRailDestination(
                icon: const Icon(Icons.font_download_outlined),
                selectedIcon: const Icon(Icons.font_download),
                label: Text(l.navLibrary),
              ),
              NavigationRailDestination(
                icon: const Icon(Icons.settings_outlined),
                selectedIcon: const Icon(Icons.settings),
                label: Text(l.navSettings),
              ),
            ],
          ),
          const VerticalDivider(width: 1),
          Expanded(
            child: IndexedStack(
              index: _page,
              children: const [JobsPage(), LibraryPage(), SettingsPage()],
            ),
          ),
        ],
      ),
    );
  }
}
