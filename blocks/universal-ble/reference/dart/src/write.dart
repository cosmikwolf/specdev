// write reference — acknowledged write (withResponse: true). Payloads
// are List<int>.
import 'package:universal_ble/universal_ble.dart';

Future<void> writeSetpoint(
    BleCharacteristic characteristic, List<int> value) async {
  await characteristic.write(value, withResponse: true);
}
