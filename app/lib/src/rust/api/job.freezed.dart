// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'job.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$RunEvent {





@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is RunEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'RunEvent()';
}


}

/// @nodoc
class $RunEventCopyWith<$Res>  {
$RunEventCopyWith(RunEvent _, $Res Function(RunEvent) __);
}


/// Adds pattern-matching-related methods to [RunEvent].
extension RunEventPatterns on RunEvent {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( RunEvent_Stage value)?  stage,TResult Function( RunEvent_FontReady value)?  fontReady,TResult Function( RunEvent_Progress value)?  progress,TResult Function( RunEvent_Warning value)?  warning,TResult Function( RunEvent_Done value)?  done,TResult Function( RunEvent_Failed value)?  failed,TResult Function( RunEvent_Skipped value)?  skipped,TResult Function( RunEvent_Finished value)?  finished,required TResult orElse(),}){
final _that = this;
switch (_that) {
case RunEvent_Stage() when stage != null:
return stage(_that);case RunEvent_FontReady() when fontReady != null:
return fontReady(_that);case RunEvent_Progress() when progress != null:
return progress(_that);case RunEvent_Warning() when warning != null:
return warning(_that);case RunEvent_Done() when done != null:
return done(_that);case RunEvent_Failed() when failed != null:
return failed(_that);case RunEvent_Skipped() when skipped != null:
return skipped(_that);case RunEvent_Finished() when finished != null:
return finished(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( RunEvent_Stage value)  stage,required TResult Function( RunEvent_FontReady value)  fontReady,required TResult Function( RunEvent_Progress value)  progress,required TResult Function( RunEvent_Warning value)  warning,required TResult Function( RunEvent_Done value)  done,required TResult Function( RunEvent_Failed value)  failed,required TResult Function( RunEvent_Skipped value)  skipped,required TResult Function( RunEvent_Finished value)  finished,}){
final _that = this;
switch (_that) {
case RunEvent_Stage():
return stage(_that);case RunEvent_FontReady():
return fontReady(_that);case RunEvent_Progress():
return progress(_that);case RunEvent_Warning():
return warning(_that);case RunEvent_Done():
return done(_that);case RunEvent_Failed():
return failed(_that);case RunEvent_Skipped():
return skipped(_that);case RunEvent_Finished():
return finished(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( RunEvent_Stage value)?  stage,TResult? Function( RunEvent_FontReady value)?  fontReady,TResult? Function( RunEvent_Progress value)?  progress,TResult? Function( RunEvent_Warning value)?  warning,TResult? Function( RunEvent_Done value)?  done,TResult? Function( RunEvent_Failed value)?  failed,TResult? Function( RunEvent_Skipped value)?  skipped,TResult? Function( RunEvent_Finished value)?  finished,}){
final _that = this;
switch (_that) {
case RunEvent_Stage() when stage != null:
return stage(_that);case RunEvent_FontReady() when fontReady != null:
return fontReady(_that);case RunEvent_Progress() when progress != null:
return progress(_that);case RunEvent_Warning() when warning != null:
return warning(_that);case RunEvent_Done() when done != null:
return done(_that);case RunEvent_Failed() when failed != null:
return failed(_that);case RunEvent_Skipped() when skipped != null:
return skipped(_that);case RunEvent_Finished() when finished != null:
return finished(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( int episode,  RunStage stage)?  stage,TResult Function( int episode,  String font,  String attachment,  BigInt bytes)?  fontReady,TResult Function( int episode,  int percent)?  progress,TResult Function( int? episode,  String message)?  warning,TResult Function( int episode,  String output,  List<String> warnings)?  done,TResult Function( int episode,  String error)?  failed,TResult Function( int episode,  String reason)?  skipped,TResult Function( int succeeded,  int failed,  bool cancelled)?  finished,required TResult orElse(),}) {final _that = this;
switch (_that) {
case RunEvent_Stage() when stage != null:
return stage(_that.episode,_that.stage);case RunEvent_FontReady() when fontReady != null:
return fontReady(_that.episode,_that.font,_that.attachment,_that.bytes);case RunEvent_Progress() when progress != null:
return progress(_that.episode,_that.percent);case RunEvent_Warning() when warning != null:
return warning(_that.episode,_that.message);case RunEvent_Done() when done != null:
return done(_that.episode,_that.output,_that.warnings);case RunEvent_Failed() when failed != null:
return failed(_that.episode,_that.error);case RunEvent_Skipped() when skipped != null:
return skipped(_that.episode,_that.reason);case RunEvent_Finished() when finished != null:
return finished(_that.succeeded,_that.failed,_that.cancelled);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( int episode,  RunStage stage)  stage,required TResult Function( int episode,  String font,  String attachment,  BigInt bytes)  fontReady,required TResult Function( int episode,  int percent)  progress,required TResult Function( int? episode,  String message)  warning,required TResult Function( int episode,  String output,  List<String> warnings)  done,required TResult Function( int episode,  String error)  failed,required TResult Function( int episode,  String reason)  skipped,required TResult Function( int succeeded,  int failed,  bool cancelled)  finished,}) {final _that = this;
switch (_that) {
case RunEvent_Stage():
return stage(_that.episode,_that.stage);case RunEvent_FontReady():
return fontReady(_that.episode,_that.font,_that.attachment,_that.bytes);case RunEvent_Progress():
return progress(_that.episode,_that.percent);case RunEvent_Warning():
return warning(_that.episode,_that.message);case RunEvent_Done():
return done(_that.episode,_that.output,_that.warnings);case RunEvent_Failed():
return failed(_that.episode,_that.error);case RunEvent_Skipped():
return skipped(_that.episode,_that.reason);case RunEvent_Finished():
return finished(_that.succeeded,_that.failed,_that.cancelled);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( int episode,  RunStage stage)?  stage,TResult? Function( int episode,  String font,  String attachment,  BigInt bytes)?  fontReady,TResult? Function( int episode,  int percent)?  progress,TResult? Function( int? episode,  String message)?  warning,TResult? Function( int episode,  String output,  List<String> warnings)?  done,TResult? Function( int episode,  String error)?  failed,TResult? Function( int episode,  String reason)?  skipped,TResult? Function( int succeeded,  int failed,  bool cancelled)?  finished,}) {final _that = this;
switch (_that) {
case RunEvent_Stage() when stage != null:
return stage(_that.episode,_that.stage);case RunEvent_FontReady() when fontReady != null:
return fontReady(_that.episode,_that.font,_that.attachment,_that.bytes);case RunEvent_Progress() when progress != null:
return progress(_that.episode,_that.percent);case RunEvent_Warning() when warning != null:
return warning(_that.episode,_that.message);case RunEvent_Done() when done != null:
return done(_that.episode,_that.output,_that.warnings);case RunEvent_Failed() when failed != null:
return failed(_that.episode,_that.error);case RunEvent_Skipped() when skipped != null:
return skipped(_that.episode,_that.reason);case RunEvent_Finished() when finished != null:
return finished(_that.succeeded,_that.failed,_that.cancelled);case _:
  return null;

}
}

}

/// @nodoc


class RunEvent_Stage extends RunEvent {
  const RunEvent_Stage({required this.episode, required this.stage}): super._();
  

 final  int episode;
 final  RunStage stage;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RunEvent_StageCopyWith<RunEvent_Stage> get copyWith => _$RunEvent_StageCopyWithImpl<RunEvent_Stage>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is RunEvent_Stage&&(identical(other.episode, episode) || other.episode == episode)&&(identical(other.stage, stage) || other.stage == stage));
}


@override
int get hashCode {
    return Object.hash(runtimeType,episode,stage);
}

@override
String toString() {
    return 'RunEvent.stage(episode: $episode, stage: $stage)';
}


}

/// @nodoc
abstract mixin class $RunEvent_StageCopyWith<$Res> implements $RunEventCopyWith<$Res> {
  factory $RunEvent_StageCopyWith(RunEvent_Stage value, $Res Function(RunEvent_Stage) _then) = _$RunEvent_StageCopyWithImpl;
@useResult
$Res call({
 int episode, RunStage stage
});




}
/// @nodoc
class _$RunEvent_StageCopyWithImpl<$Res>
    implements $RunEvent_StageCopyWith<$Res> {
  _$RunEvent_StageCopyWithImpl(this._self, this._then);

  final RunEvent_Stage _self;
  final $Res Function(RunEvent_Stage) _then;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? episode = null,Object? stage = null,}) {
  return _then(RunEvent_Stage(
episode: null == episode ? _self.episode : episode // ignore: cast_nullable_to_non_nullable
as int,stage: null == stage ? _self.stage : stage // ignore: cast_nullable_to_non_nullable
as RunStage,
  ));
}


}

/// @nodoc


class RunEvent_FontReady extends RunEvent {
  const RunEvent_FontReady({required this.episode, required this.font, required this.attachment, required this.bytes}): super._();
  

 final  int episode;
 final  String font;
 final  String attachment;
 final  BigInt bytes;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RunEvent_FontReadyCopyWith<RunEvent_FontReady> get copyWith => _$RunEvent_FontReadyCopyWithImpl<RunEvent_FontReady>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is RunEvent_FontReady&&(identical(other.episode, episode) || other.episode == episode)&&(identical(other.font, font) || other.font == font)&&(identical(other.attachment, attachment) || other.attachment == attachment)&&(identical(other.bytes, bytes) || other.bytes == bytes));
}


@override
int get hashCode {
    return Object.hash(runtimeType,episode,font,attachment,bytes);
}

@override
String toString() {
    return 'RunEvent.fontReady(episode: $episode, font: $font, attachment: $attachment, bytes: $bytes)';
}


}

/// @nodoc
abstract mixin class $RunEvent_FontReadyCopyWith<$Res> implements $RunEventCopyWith<$Res> {
  factory $RunEvent_FontReadyCopyWith(RunEvent_FontReady value, $Res Function(RunEvent_FontReady) _then) = _$RunEvent_FontReadyCopyWithImpl;
@useResult
$Res call({
 int episode, String font, String attachment, BigInt bytes
});




}
/// @nodoc
class _$RunEvent_FontReadyCopyWithImpl<$Res>
    implements $RunEvent_FontReadyCopyWith<$Res> {
  _$RunEvent_FontReadyCopyWithImpl(this._self, this._then);

  final RunEvent_FontReady _self;
  final $Res Function(RunEvent_FontReady) _then;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? episode = null,Object? font = null,Object? attachment = null,Object? bytes = null,}) {
  return _then(RunEvent_FontReady(
episode: null == episode ? _self.episode : episode // ignore: cast_nullable_to_non_nullable
as int,font: null == font ? _self.font : font // ignore: cast_nullable_to_non_nullable
as String,attachment: null == attachment ? _self.attachment : attachment // ignore: cast_nullable_to_non_nullable
as String,bytes: null == bytes ? _self.bytes : bytes // ignore: cast_nullable_to_non_nullable
as BigInt,
  ));
}


}

/// @nodoc


class RunEvent_Progress extends RunEvent {
  const RunEvent_Progress({required this.episode, required this.percent}): super._();
  

 final  int episode;
 final  int percent;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RunEvent_ProgressCopyWith<RunEvent_Progress> get copyWith => _$RunEvent_ProgressCopyWithImpl<RunEvent_Progress>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is RunEvent_Progress&&(identical(other.episode, episode) || other.episode == episode)&&(identical(other.percent, percent) || other.percent == percent));
}


@override
int get hashCode {
    return Object.hash(runtimeType,episode,percent);
}

@override
String toString() {
    return 'RunEvent.progress(episode: $episode, percent: $percent)';
}


}

/// @nodoc
abstract mixin class $RunEvent_ProgressCopyWith<$Res> implements $RunEventCopyWith<$Res> {
  factory $RunEvent_ProgressCopyWith(RunEvent_Progress value, $Res Function(RunEvent_Progress) _then) = _$RunEvent_ProgressCopyWithImpl;
@useResult
$Res call({
 int episode, int percent
});




}
/// @nodoc
class _$RunEvent_ProgressCopyWithImpl<$Res>
    implements $RunEvent_ProgressCopyWith<$Res> {
  _$RunEvent_ProgressCopyWithImpl(this._self, this._then);

  final RunEvent_Progress _self;
  final $Res Function(RunEvent_Progress) _then;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? episode = null,Object? percent = null,}) {
  return _then(RunEvent_Progress(
episode: null == episode ? _self.episode : episode // ignore: cast_nullable_to_non_nullable
as int,percent: null == percent ? _self.percent : percent // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class RunEvent_Warning extends RunEvent {
  const RunEvent_Warning({this.episode, required this.message}): super._();
  

 final  int? episode;
 final  String message;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RunEvent_WarningCopyWith<RunEvent_Warning> get copyWith => _$RunEvent_WarningCopyWithImpl<RunEvent_Warning>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is RunEvent_Warning&&(identical(other.episode, episode) || other.episode == episode)&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode {
    return Object.hash(runtimeType,episode,message);
}

@override
String toString() {
    return 'RunEvent.warning(episode: $episode, message: $message)';
}


}

/// @nodoc
abstract mixin class $RunEvent_WarningCopyWith<$Res> implements $RunEventCopyWith<$Res> {
  factory $RunEvent_WarningCopyWith(RunEvent_Warning value, $Res Function(RunEvent_Warning) _then) = _$RunEvent_WarningCopyWithImpl;
@useResult
$Res call({
 int? episode, String message
});




}
/// @nodoc
class _$RunEvent_WarningCopyWithImpl<$Res>
    implements $RunEvent_WarningCopyWith<$Res> {
  _$RunEvent_WarningCopyWithImpl(this._self, this._then);

  final RunEvent_Warning _self;
  final $Res Function(RunEvent_Warning) _then;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? episode = freezed,Object? message = null,}) {
  return _then(RunEvent_Warning(
episode: freezed == episode ? _self.episode : episode // ignore: cast_nullable_to_non_nullable
as int?,message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class RunEvent_Done extends RunEvent {
  const RunEvent_Done({required this.episode, required this.output, required  List<String> warnings}): _warnings = warnings,super._();
  

 final  int episode;
 final  String output;
 final  List<String> _warnings;
 List<String> get warnings {
  if (_warnings is EqualUnmodifiableListView) return _warnings;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_warnings);
}


/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RunEvent_DoneCopyWith<RunEvent_Done> get copyWith => _$RunEvent_DoneCopyWithImpl<RunEvent_Done>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is RunEvent_Done&&(identical(other.episode, episode) || other.episode == episode)&&(identical(other.output, output) || other.output == output)&&const DeepCollectionEquality().equals(other.warnings, _warnings));
}


@override
int get hashCode {
    return Object.hash(runtimeType,episode,output,const DeepCollectionEquality().hash(_warnings));
}

@override
String toString() {
    return 'RunEvent.done(episode: $episode, output: $output, warnings: $warnings)';
}


}

/// @nodoc
abstract mixin class $RunEvent_DoneCopyWith<$Res> implements $RunEventCopyWith<$Res> {
  factory $RunEvent_DoneCopyWith(RunEvent_Done value, $Res Function(RunEvent_Done) _then) = _$RunEvent_DoneCopyWithImpl;
@useResult
$Res call({
 int episode, String output, List<String> warnings
});




}
/// @nodoc
class _$RunEvent_DoneCopyWithImpl<$Res>
    implements $RunEvent_DoneCopyWith<$Res> {
  _$RunEvent_DoneCopyWithImpl(this._self, this._then);

  final RunEvent_Done _self;
  final $Res Function(RunEvent_Done) _then;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? episode = null,Object? output = null,Object? warnings = null,}) {
  return _then(RunEvent_Done(
episode: null == episode ? _self.episode : episode // ignore: cast_nullable_to_non_nullable
as int,output: null == output ? _self.output : output // ignore: cast_nullable_to_non_nullable
as String,warnings: null == warnings ? _self._warnings : warnings // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class RunEvent_Failed extends RunEvent {
  const RunEvent_Failed({required this.episode, required this.error}): super._();
  

 final  int episode;
 final  String error;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RunEvent_FailedCopyWith<RunEvent_Failed> get copyWith => _$RunEvent_FailedCopyWithImpl<RunEvent_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is RunEvent_Failed&&(identical(other.episode, episode) || other.episode == episode)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode {
    return Object.hash(runtimeType,episode,error);
}

@override
String toString() {
    return 'RunEvent.failed(episode: $episode, error: $error)';
}


}

/// @nodoc
abstract mixin class $RunEvent_FailedCopyWith<$Res> implements $RunEventCopyWith<$Res> {
  factory $RunEvent_FailedCopyWith(RunEvent_Failed value, $Res Function(RunEvent_Failed) _then) = _$RunEvent_FailedCopyWithImpl;
@useResult
$Res call({
 int episode, String error
});




}
/// @nodoc
class _$RunEvent_FailedCopyWithImpl<$Res>
    implements $RunEvent_FailedCopyWith<$Res> {
  _$RunEvent_FailedCopyWithImpl(this._self, this._then);

  final RunEvent_Failed _self;
  final $Res Function(RunEvent_Failed) _then;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? episode = null,Object? error = null,}) {
  return _then(RunEvent_Failed(
episode: null == episode ? _self.episode : episode // ignore: cast_nullable_to_non_nullable
as int,error: null == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class RunEvent_Skipped extends RunEvent {
  const RunEvent_Skipped({required this.episode, required this.reason}): super._();
  

 final  int episode;
 final  String reason;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RunEvent_SkippedCopyWith<RunEvent_Skipped> get copyWith => _$RunEvent_SkippedCopyWithImpl<RunEvent_Skipped>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is RunEvent_Skipped&&(identical(other.episode, episode) || other.episode == episode)&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode {
    return Object.hash(runtimeType,episode,reason);
}

@override
String toString() {
    return 'RunEvent.skipped(episode: $episode, reason: $reason)';
}


}

/// @nodoc
abstract mixin class $RunEvent_SkippedCopyWith<$Res> implements $RunEventCopyWith<$Res> {
  factory $RunEvent_SkippedCopyWith(RunEvent_Skipped value, $Res Function(RunEvent_Skipped) _then) = _$RunEvent_SkippedCopyWithImpl;
@useResult
$Res call({
 int episode, String reason
});




}
/// @nodoc
class _$RunEvent_SkippedCopyWithImpl<$Res>
    implements $RunEvent_SkippedCopyWith<$Res> {
  _$RunEvent_SkippedCopyWithImpl(this._self, this._then);

  final RunEvent_Skipped _self;
  final $Res Function(RunEvent_Skipped) _then;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? episode = null,Object? reason = null,}) {
  return _then(RunEvent_Skipped(
episode: null == episode ? _self.episode : episode // ignore: cast_nullable_to_non_nullable
as int,reason: null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class RunEvent_Finished extends RunEvent {
  const RunEvent_Finished({required this.succeeded, required this.failed, required this.cancelled}): super._();
  

 final  int succeeded;
 final  int failed;
 final  bool cancelled;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RunEvent_FinishedCopyWith<RunEvent_Finished> get copyWith => _$RunEvent_FinishedCopyWithImpl<RunEvent_Finished>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is RunEvent_Finished&&(identical(other.succeeded, succeeded) || other.succeeded == succeeded)&&(identical(other.failed, failed) || other.failed == failed)&&(identical(other.cancelled, cancelled) || other.cancelled == cancelled));
}


@override
int get hashCode {
    return Object.hash(runtimeType,succeeded,failed,cancelled);
}

@override
String toString() {
    return 'RunEvent.finished(succeeded: $succeeded, failed: $failed, cancelled: $cancelled)';
}


}

/// @nodoc
abstract mixin class $RunEvent_FinishedCopyWith<$Res> implements $RunEventCopyWith<$Res> {
  factory $RunEvent_FinishedCopyWith(RunEvent_Finished value, $Res Function(RunEvent_Finished) _then) = _$RunEvent_FinishedCopyWithImpl;
@useResult
$Res call({
 int succeeded, int failed, bool cancelled
});




}
/// @nodoc
class _$RunEvent_FinishedCopyWithImpl<$Res>
    implements $RunEvent_FinishedCopyWith<$Res> {
  _$RunEvent_FinishedCopyWithImpl(this._self, this._then);

  final RunEvent_Finished _self;
  final $Res Function(RunEvent_Finished) _then;

/// Create a copy of RunEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? succeeded = null,Object? failed = null,Object? cancelled = null,}) {
  return _then(RunEvent_Finished(
succeeded: null == succeeded ? _self.succeeded : succeeded // ignore: cast_nullable_to_non_nullable
as int,failed: null == failed ? _self.failed : failed // ignore: cast_nullable_to_non_nullable
as int,cancelled: null == cancelled ? _self.cancelled : cancelled // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
