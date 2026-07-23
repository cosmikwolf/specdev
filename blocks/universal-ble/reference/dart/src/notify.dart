// notify reference — subscribe to notifications, then consume incoming
// values on onValueReceived. Mirrors the README worked example.
import 'dart:typed_data';

import 'package:universal_ble/universal_ble.dart';

Future<Stream<Uint8List>> subscribeBatteryLevel(
    BleCharacteristic characteristic) async {
  await characteristic.notifications.subscribe();
  return characteristic.onValueReceived;
}

Future<void> stopBatteryLevel(BleCharacteristic characteristic) async {
  await characteristic.notifications.unsubscribe();
}
