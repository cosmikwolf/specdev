// indicate reference — acknowledged-notify via indications; gate on
// isSupported when the peripheral firmware is not fully trusted.
import 'package:universal_ble/universal_ble.dart';

Future<void> subscribeAlerts(BleCharacteristic characteristic) async {
  if (characteristic.indications.isSupported) {
    await characteristic.indications.subscribe();
  }
}
