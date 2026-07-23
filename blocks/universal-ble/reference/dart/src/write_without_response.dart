// write_without_response reference — fire-and-forget write
// (withResponse: false).
import 'package:universal_ble/universal_ble.dart';

Future<void> writeStreamChunk(
    BleCharacteristic characteristic, List<int> value) async {
  await characteristic.write(value, withResponse: false);
}
