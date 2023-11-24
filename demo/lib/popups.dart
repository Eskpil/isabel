import 'package:flutter/services.dart';

class Popups {
  static const platform = MethodChannel('isabel.io/popups', JSONMethodCodec());

  Popups();

  void spawn(String name, double x, double y) {
    var data = <dynamic, dynamic>{'x': x, 'y': y};
    platform.invokeMethod(name, data);
  }
}
