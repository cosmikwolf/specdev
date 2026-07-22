// stream reference — a Rust fn taking a `StreamSink<T>` surfaces in Dart as
// a function RETURNING `Stream<T>`; the sink parameter disappears from the
// Dart signature.
import 'package:frb_reference/src/rust/api.dart';

Stream<int> counterUpTo(int limit) {
  final Stream<int> ticks = countUpTo(limit: limit);
  return ticks;
}
