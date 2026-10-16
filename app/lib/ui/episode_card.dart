import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/app_localizations.dart';
import '../src/rust/api/job.dart';
import '../state/jobs.dart';
import 'common.dart';
import 'font_picker.dart';

const _languages = ['zh-Hans', 'zh-Hant', 'en', 'ja', 'ko', 'und'];

class EpisodeCard extends ConsumerWidget {
  const EpisodeCard({super.key, required this.index});

  final int index;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l = AppLocalizations.of(context);
    final job = ref.watch(jobsProvider.select((s) => s.jobs[index]));
    final running = ref.watch(jobsProvider.select((s) => s.running));
    final notifier = ref.read(jobsProvider.notifier);
    final theme = Theme.of(context);
    final plan = job.plan;

    final subtitleLine = job.episode.subtitles.isEmpty
        ? l.noSubtitles
        : [
            l.subtitlesCount(job.episode.subtitles.length),
            if (plan != null) l.fontsSummary(plan.fonts.length, plan.missing.length),
            if (job.analyzing) l.analyzing,
          ].join(' · ');

    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      clipBehavior: Clip.antiAlias,
      child: Column(
        children: [
          ExpansionTile(
            key: ValueKey(job.episode.mkv),
            leading: _StatusIcon(job: job),
            title: Text(fileName(job.episode.mkv), maxLines: 1, overflow: TextOverflow.ellipsis),
            subtitle: Text(
              job.status == JobStatus.running && job.stage != null
                  ? '${stageLabel(l, job.stage!)} ${job.stage == RunStage.muxing ? '${job.percent}%' : ''}'
                  : subtitleLine,
            ),
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (job.output != null)
                  IconButton(
                    tooltip: l.openFolder,
                    icon: const Icon(Icons.folder_outlined),
                    onPressed: () => reveal(job.output!),
                  ),
                if (!running)
                  IconButton(
                    tooltip: l.remove,
                    icon: const Icon(Icons.close),
                    onPressed: () => notifier.remove(index),
                  ),
                const Icon(Icons.expand_more),
              ],
            ),
            childrenPadding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
            expandedCrossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (job.error != null)
                _Message(
                  icon: Icons.error_outline,
                  color: theme.colorScheme.error,
                  text: job.error!,
                ),
              if (job.output != null) _Message(icon: Icons.check, text: job.output!),
              ..._subtitles(context, ref, job),
              if (plan != null) ..._fonts(context, ref, plan),
              if (job.warnings.isNotEmpty || (plan?.warnings.isNotEmpty ?? false)) ...[
                _Header(l.warnings),
                for (final w in {...?plan?.warnings, ...job.warnings})
                  _Message(icon: Icons.warning_amber, color: theme.colorScheme.tertiary, text: w),
              ],
            ],
          ),
          if (job.status == JobStatus.running)
            LinearProgressIndicator(
              value: job.stage == RunStage.muxing ? job.percent / 100 : null,
              minHeight: 3,
            ),
        ],
      ),
    );
  }

  List<Widget> _subtitles(BuildContext context, WidgetRef ref, Job job) {
    final l = AppLocalizations.of(context);
    final encodings = {
      for (final s in job.plan?.subtitles ?? <SubtitleSummary>[]) s.path: s.encoding,
    };
    final editable = job.status == JobStatus.pending;
    return [
      for (var i = 0; i < job.episode.subtitles.length; i++)
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: Row(
            children: [
              const Icon(Icons.subtitles_outlined, size: 18),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  fileName(job.episode.subtitles[i].path) +
                      (encodings[job.episode.subtitles[i].path] != null
                          ? '  (${encodings[job.episode.subtitles[i].path]})'
                          : ''),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              DropdownMenu<String>(
                width: 140,
                enabled: editable,
                initialSelection: job.episode.subtitles[i].language,
                label: Text(l.language),
                inputDecorationTheme: const InputDecorationTheme(isDense: true),
                dropdownMenuEntries: [
                  for (final lang in {..._languages, job.episode.subtitles[i].language})
                    DropdownMenuEntry(value: lang, label: lang),
                ],
                onSelected: (value) {
                  if (value != null) ref.read(jobsProvider.notifier).setLanguage(index, i, value);
                },
              ),
            ],
          ),
        ),
    ];
  }

  List<Widget> _fonts(BuildContext context, WidgetRef ref, PlanSummary plan) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final overrides = ref.watch(jobsProvider.select((s) => s.overrides));
    final notifier = ref.read(jobsProvider.notifier);
    final running = ref.watch(jobsProvider.select((s) => s.running));

    Future<void> choose(String name) async {
      final choice = await pickFont(context, name);
      if (choice != null) await notifier.setOverride(name, choice);
    }

    Widget overrideMenu(String name) => PopupMenuButton<String>(
      enabled: !running,
      tooltip: l.changeFont,
      icon: const Icon(Icons.more_horiz, size: 18),
      itemBuilder: (_) => [
        PopupMenuItem(value: 'change', child: Text(l.changeFont)),
        if (overrides.containsKey(name))
          PopupMenuItem(value: 'reset', child: Text(l.removeOverride)),
      ],
      onSelected: (v) => v == 'change' ? choose(name) : notifier.setOverride(name, null),
    );

    return [
      if (plan.missing.isNotEmpty) ...[
        _Header(l.missingFonts),
        for (final m in plan.missing)
          Row(
            children: [
              Icon(Icons.cancel_outlined, size: 18, color: theme.colorScheme.error),
              const SizedBox(width: 8),
              Expanded(
                child: Text.rich(
                  TextSpan(
                    text: '${m.name}${styleLabel(m.weight, m.italic)}',
                    children: [
                      TextSpan(
                        text: '  ${m.styleOnly ? '${l.styleOnly} · ' : ''}${m.uses.join('  ')}',
                        style: TextStyle(color: theme.colorScheme.outline),
                      ),
                    ],
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              TextButton(
                onPressed: running ? null : () => choose(m.name),
                child: Text(l.chooseFont),
              ),
            ],
          ),
      ],
      if (plan.fonts.isNotEmpty) ...[
        _Header(l.fonts),
        for (final f in plan.fonts)
          Row(
            children: [
              Icon(
                f.ambiguous ? Icons.help_outline : Icons.check_circle_outline,
                size: 18,
                color: f.ambiguous ? theme.colorScheme.tertiary : theme.colorScheme.primary,
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Text.rich(
                  TextSpan(
                    text: f.name,
                    children: [
                      TextSpan(
                        text:
                            '  → ${fileName(f.path)}${f.faceIndex > 0 ? '#${f.faceIndex}' : ''}'
                            ' · ${l.chars(f.characters)}'
                            '${f.ambiguous ? ' · ${l.ambiguous}' : ''}'
                            '${f.fallbackName != null ? ' · ${l.matchedAs(f.fallbackName!)}' : ''}'
                            '${overrides.containsKey(f.name) ? ' · ${l.manual}' : ''}',
                        style: TextStyle(color: theme.colorScheme.outline),
                      ),
                    ],
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              overrideMenu(f.name),
            ],
          ),
      ],
      if (plan.glyphGaps.isNotEmpty) ...[
        _Header(l.missingGlyphs),
        for (final g in plan.glyphGaps)
          _Message(
            icon: Icons.text_format,
            color: theme.colorScheme.tertiary,
            text: '${g.font}: ${g.characters}',
          ),
      ],
    ];
  }
}

class _StatusIcon extends StatelessWidget {
  const _StatusIcon({required this.job});

  final Job job;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return switch (job.status) {
      JobStatus.running => const SizedBox.square(
        dimension: 22,
        child: CircularProgressIndicator(strokeWidth: 2.5),
      ),
      JobStatus.queued => Icon(Icons.schedule, color: scheme.outline),
      JobStatus.done => Icon(Icons.check_circle, color: scheme.primary),
      JobStatus.failed => Icon(Icons.error, color: scheme.error),
      JobStatus.skipped => Icon(Icons.remove_circle_outline, color: scheme.outline),
      JobStatus.pending when job.analyzing => const SizedBox.square(
        dimension: 22,
        child: CircularProgressIndicator(strokeWidth: 2),
      ),
      JobStatus.pending =>
        job.hasIssues
            ? Icon(Icons.warning_amber_rounded, color: scheme.tertiary)
            : Icon(Icons.movie_outlined, color: scheme.secondary),
    };
  }
}

class _Header extends StatelessWidget {
  const _Header(this.text);

  final String text;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 12, bottom: 4),
    child: Text(text, style: Theme.of(context).textTheme.labelLarge),
  );
}

class _Message extends StatelessWidget {
  const _Message({required this.icon, required this.text, this.color});

  final IconData icon;
  final String text;
  final Color? color;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 4),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Icon(icon, size: 18, color: color),
        const SizedBox(width: 8),
        Expanded(child: SelectableText(text)),
      ],
    ),
  );
}
