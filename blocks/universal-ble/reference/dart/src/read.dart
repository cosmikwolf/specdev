// read reference — resolve the characteristic through its declared
// service (characteristic UUID positional, service UUID as the required
// named parameter), then one-shot read.
import 'dart:typed_data';

import 'package:universal_ble/universal_ble.dart';

const String batteryServiceUuid = '180F';
const String batteryLevelCharacteristicUuid = '2A19';

Future<Uint8List> readBatteryLevel(BleDevice device) async {
  final BleCharacteristic characteristic = await device.getCharacteristic(
      batteryLevelCharacteristicUuid,
      service: batteryServiceUuid);
  final Uint8List value = await characteristic.read();
  return value;
}
