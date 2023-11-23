import 'package:flutter/services.dart';

class SerialEvent {
  int serial;

  SerialEvent({required this.serial});

  factory SerialEvent.fromJSON(dynamic json) {
    return SerialEvent(serial: json['serial']);
  }
}

class Decorations {
  static const platform =
      MethodChannel('isabel.io/decorations', JSONMethodCodec());

  SerialEvent? last_move;

  Decorations() {
    platform.setMethodCallHandler((call) async {
      if (call.method == "move") {
        final event = SerialEvent.fromJSON(call.arguments);
        last_move = event;
      }
    });
  }

  void initiateMove(SerialEvent event) {
    var data = <dynamic, dynamic>{'serial': event.serial};
    platform.invokeMethod("initiate_move", data);
  }
}
