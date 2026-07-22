// sync_call reference — a `#[frb(sync)]` Rust fn surfaces as a plain
// synchronous Dart function with required named parameters.
import 'package:frb_reference/src/rust/api.dart';

String greetUser(String name) {
  final String message = greet(name: name);
  return message;
}
