import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../src/rust/api/job.dart';
import 'library.dart';
import 'settings.dart';

enum JobStatus { pending, queued, running, done, failed, skipped }

class Job {
  const Job({
    required this.episode,
    this.plan,
    this.analyzing = false,
    this.status = JobStatus.pending,
    this.stage,
    this.percent = 0,
    this.output,
    this.error,
    this.warnings = const [],
  });

  final EpisodeDto episode;
  final PlanSummary? plan;
  final bool analyzing;
  final JobStatus status;
  final RunStage? stage;
  final int percent;
  final String? output;
  final String? error;
  final List<String> warnings;

  bool get hasIssues =>
      episode.subtitles.isEmpty ||
      (plan != null && (plan!.missing.isNotEmpty || plan!.glyphGaps.isNotEmpty));

  Job copyWith({
    EpisodeDto? episode,
    PlanSummary? Function()? plan,
    bool? analyzing,
    JobStatus? status,
    RunStage? Function()? stage,
    int? percent,
    String? Function()? output,
    String? Function()? error,
    List<String>? warnings,
  }) {
    return Job(
      episode: episode ?? this.episode,
      plan: plan != null ? plan() : this.plan,
      analyzing: analyzing ?? this.analyzing,
      status: status ?? this.status,
      stage: stage != null ? stage() : this.stage,
      percent: percent ?? this.percent,
      output: output != null ? output() : this.output,
      error: error != null ? error() : this.error,
      warnings: warnings ?? this.warnings,
    );
  }
}

class JobsState {
  const JobsState({
    this.jobs = const [],
    this.overrides = const {},
    this.running = false,
    this.log = const [],
    this.summary,
  });

  final List<Job> jobs;

  /// Font name -> chosen face, applied to every episode.
  final Map<String, FontOverride> overrides;
  final bool running;
  final List<String> log;

  /// Result line of the last finished run.
  final String? summary;

  JobsState copyWith({
    List<Job>? jobs,
    Map<String, FontOverride>? overrides,
    bool? running,
    List<String>? log,
    String? Function()? summary,
  }) {
    return JobsState(
      jobs: jobs ?? this.jobs,
      overrides: overrides ?? this.overrides,
      running: running ?? this.running,
      log: log ?? this.log,
      summary: summary != null ? summary() : this.summary,
    );
  }
}

class JobsNotifier extends Notifier<JobsState> {
  CancelToken? _cancel;

  @override
  JobsState build() {
    // Re-run preflight when the library finishes loading.
    ref.listen(libraryProvider.select((s) => s.library), (_, library) {
      if (library != null) analyzeAll();
    });
    // Matching options change the preflight result.
    ref.listen(settingsProvider, (_, next) {
      if (next.hasValue) analyzeAll();
    });
    return const JobsState();
  }

  void _log(String line) {
    final log = [...state.log, line];
    state = state.copyWith(log: log.length > 2000 ? log.sublist(log.length - 2000) : log);
  }

  void _updateJob(int index, Job Function(Job) update) {
    if (index < 0 || index >= state.jobs.length) return;
    final jobs = [...state.jobs];
    jobs[index] = update(jobs[index]);
    state = state.copyWith(jobs: jobs);
  }

  Future<void> addPaths(List<String> paths) async {
    if (paths.isEmpty || state.running) return;
    final settings = await ref.read(settingsProvider.future);
    try {
      final found = await discover(paths: paths, settings: settings);
      final known = state.jobs.map((j) => j.episode.mkv).toSet();
      final added = found.where((e) => !known.contains(e.mkv)).toList();
      state = state.copyWith(
        jobs: [
          ...state.jobs,
          ...added.map((e) => Job(episode: e)),
        ],
        summary: () => null,
      );
      await analyzeAll();
    } catch (e) {
      _log('$e');
    }
  }

  void remove(int index) {
    if (state.running) return;
    state = state.copyWith(jobs: [...state.jobs]..removeAt(index));
  }

  void clear() {
    if (state.running) return;
    state = state.copyWith(jobs: [], summary: () => null);
  }

  Future<void> setLanguage(int job, int subtitle, String language) async {
    _updateJob(job, (j) {
      final subs = [...j.episode.subtitles];
      subs[subtitle] = SubtitleDto(path: subs[subtitle].path, language: language);
      return j.copyWith(
        episode: EpisodeDto(mkv: j.episode.mkv, subtitles: subs),
      );
    });
  }

  Future<void> setOverride(String name, FontOverride? choice) async {
    final overrides = {...state.overrides};
    if (choice == null) {
      overrides.remove(name);
    } else {
      overrides[name] = choice;
    }
    state = state.copyWith(overrides: overrides);
    await analyzeAll();
  }

  Future<void> analyzeAll() async {
    final library = ref.read(libraryProvider).library;
    if (library == null || state.running) return;
    final settings = await ref.read(settingsProvider.future);
    final overrides = state.overrides.values.toList();
    for (var i = 0; i < state.jobs.length; i++) {
      final job = state.jobs[i];
      if (job.status != JobStatus.pending) continue;
      _updateJob(i, (j) => j.copyWith(analyzing: true));
      try {
        final plan = await analyze(
          library_: library,
          episode: job.episode,
          settings: settings,
          overridesList: overrides,
        );
        // The list may have changed while analyzing.
        final index = state.jobs.indexWhere((j) => j.episode.mkv == job.episode.mkv);
        _updateJob(index, (j) => j.copyWith(plan: () => plan, analyzing: false));
      } catch (e) {
        _updateJob(i, (j) => j.copyWith(analyzing: false, error: () => '$e'));
      }
    }
  }

  Future<void> start() async {
    final library = ref.read(libraryProvider).library;
    if (library == null || state.running || state.jobs.isEmpty) return;
    final settings = await ref.read(settingsProvider.future);
    final indices = [
      for (var i = 0; i < state.jobs.length; i++)
        if (state.jobs[i].status != JobStatus.done) i,
    ];
    if (indices.isEmpty) return;
    for (final i in indices) {
      _updateJob(
        i,
        (j) => j.copyWith(
          status: JobStatus.queued,
          stage: () => null,
          percent: 0,
          error: () => null,
          output: () => null,
        ),
      );
    }
    _cancel = CancelToken();
    state = state.copyWith(running: true, summary: () => null);

    final stream = runBatch(
      library_: library,
      episodes: [for (final i in indices) state.jobs[i].episode],
      settings: settings,
      overridesList: state.overrides.values.toList(),
      cancel: _cancel!,
    );
    String name(int e) => state.jobs[indices[e]].episode.mkv.split(RegExp(r'[\\/]')).last;
    await for (final event in stream) {
      switch (event) {
        case RunEvent_Stage(:final episode, :final stage):
          _updateJob(
            indices[episode],
            (j) => j.copyWith(status: JobStatus.running, stage: () => stage),
          );
        case RunEvent_Progress(:final episode, :final percent):
          _updateJob(indices[episode], (j) => j.copyWith(percent: percent));
        case RunEvent_FontReady(:final episode, :final font, :final attachment, :final bytes):
          _log('${name(episode)}: $font → $attachment (${(bytes.toDouble() / 1024).toStringAsFixed(1)} KiB)');
        case RunEvent_Warning(:final episode, :final message):
          _log(episode == null ? message : '${name(episode)}: $message');
        case RunEvent_Done(:final episode, :final output, :final warnings):
          _updateJob(
            indices[episode],
            (j) => j.copyWith(
              status: JobStatus.done,
              percent: 100,
              output: () => output,
              warnings: warnings,
            ),
          );
          _log('${name(episode)}: → $output');
        case RunEvent_Failed(:final episode, :final error):
          _updateJob(
            indices[episode],
            (j) => j.copyWith(status: JobStatus.failed, error: () => error),
          );
          _log('${name(episode)}: $error');
        case RunEvent_Skipped(:final episode, :final reason):
          _updateJob(
            indices[episode],
            (j) => j.copyWith(status: JobStatus.skipped, error: () => reason),
          );
        case RunEvent_Finished(:final succeeded, :final failed, :final cancelled):
          state = state.copyWith(
            summary: () =>
                cancelled ? 'cancelled:$succeeded:$failed' : 'finished:$succeeded:$failed',
          );
      }
    }
    // Episodes still queued after a cancel go back to pending.
    final jobs = [
      for (final j in state.jobs)
        j.status == JobStatus.queued || j.status == JobStatus.running
            ? j.copyWith(status: JobStatus.pending, stage: () => null, percent: 0)
            : j,
    ];
    state = state.copyWith(jobs: jobs, running: false);
    _cancel = null;
  }

  void cancel() => _cancel?.cancel();
}

final jobsProvider = NotifierProvider<JobsNotifier, JobsState>(JobsNotifier.new);
