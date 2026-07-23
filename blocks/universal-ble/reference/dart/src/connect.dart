// connect reference — connect on a scanned BleDevice, discover services
// before any characteristic access, and watch connectionStream for
// drops. Mirrors the README worked example.
import 'dart:async';

import 'package:universal_ble/universal_ble.dart';

Future<List<BleService>> connectAndDiscover(BleDevice device) async {
  await device.connect();
  final List<BleService> services = await device.discoverServices();
  return services;
}

StreamSubscription<bool> watchConnection(
    BleDevice device, void Function() onDrop) {
  return device.connectionStream.listen((bool connected) {
    if (!connected) {
      onDrop();
    }
  });
}
