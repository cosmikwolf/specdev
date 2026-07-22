// async_call reference — the default FRB shape: `async fn` in Rust surfaces
// as a Future and runs off the UI thread.
import 'package:frb_reference/src/rust/api.dart';

Future<int> checksumOf(List<int> data) async {
  final int sum = await checksum(data: data);
  return sum;
}
