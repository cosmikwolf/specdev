// init reference — mirrors the README worked example. The generated
// bindings are unusable until `RustLib.init()` completes; a real app awaits
// it in main() before runApp().
import 'package:frb_reference/src/rust/frb_generated.dart';

Future<void> initRust() async {
  await RustLib.init();
}
