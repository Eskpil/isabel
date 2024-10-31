import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

void sidecar_main() {}

class SidecarRequestSpawn {
  final double x, y;
  final String entry;

  const SidecarRequestSpawn(
      {required this.x, required this.y, required this.entry});

  Map<dynamic, dynamic> serialize() {
    var data = {'x': x, 'y': y, 'entry': entry};
    return data;
  }
}

class Sidecar {
  static const platform = MethodChannel('isabel/sidecar', JSONMethodCodec());

  Sidecar() {
    platform.setMethodCallHandler((call) async {
      print(call);
    });
  }

  void spawn(double x, double y, String entry) {
    final req = SidecarRequestSpawn(entry: entry, x: x, y: y);
    platform.invokeMethod('Spawn', req.serialize());
  }
}

class SerialEvent {
  int serial;

  SerialEvent({required this.serial});

  factory SerialEvent.fromJSON(dynamic json) {
    return SerialEvent(serial: json['serial']);
  }
}

class Decorations {
  static const platform =
      MethodChannel('isabel/decorations', JSONMethodCodec());

  SerialEvent? lastMove;

  Decorations() {
    platform.setMethodCallHandler((call) async {
      if (call.method == "move") {
        final event = SerialEvent.fromJSON(call.arguments);
        lastMove = event;
      }
    });
  }

  void initiateMove(SerialEvent event) {
    var data = <dynamic, dynamic>{'serial': event.serial};
    platform.invokeMethod("initiateMove", data);
  }
}

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
//      home: const MyHomePage(title: 'hva faen'),
      home: const Layer(),
    );
  }
}

class Layer extends StatelessWidget {
  const Layer({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
        color: Theme.of(context).colorScheme.inversePrimary,
        child: const Center(
          child: Text('Panel', style: TextStyle(fontSize: 12)),
        ));
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});
  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  final Decorations _decorations = Decorations();
  final Sidecar _sidecar = Sidecar();
  int count = 0;

  void _move(DragStartDetails details) {
    if (_decorations.lastMove == null) {
      return;
    }

    _decorations.initiateMove(_decorations.lastMove!);
  }

  void _onPressed(TapDownDetails details) {
    print(details.globalPosition.dx);
    print(details.globalPosition.dy);
    print('');

    _sidecar.spawn(
        details.globalPosition.dx, details.globalPosition.dy, "sidecar_main");
  }

  void _onIncrementPressed() {
    setState(() {
      count++;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            Text('Pressed $count times'),
            TextButton(
                onPressed: _onIncrementPressed, child: const Text('Increment')),
            const Text('Welcome to the isabel flutter emebdder',
                style: TextStyle(color: Colors.black)),
            GestureDetector(
                onTapDown: _onPressed,
                child: Container(
                    color: Theme.of(context).colorScheme.primary,
                    child: const Text('Open popup')))
          ],
        ),
      ),
    );
  }
}
