// scan reference — UUID constants emitted verbatim from the spec, the
// derived scan filter listing EVERY declared service, the scan stream,
// and stopping the scan once the target is found. Mirrors the README
// worked example.
import 'package:universal_ble/universal_ble.dart';

const String batteryServiceUuid = '180F';
const String batteryLevelCharacteristicUuid = '2A19';

ScanFilter gattScanFilter() {
  return ScanFilter(withServices: [batteryServiceUuid]);
}

Future<Stream<BleDevice>> scanForPeripherals() async {
  await UniversalBle.startScan(scanFilter: gattScanFilter());
  return UniversalBle.scanStream;
}

Future<void> stopScanning() async {
  await UniversalBle.stopScan();
}
