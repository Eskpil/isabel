import 'dart:async';

import 'package:flutter/material.dart';
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
      home: const MyHomePage(title: 'hva faen'),
    );
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
  void _move(DragStartDetails details) {
    if (_decorations.lastMove == null) {
      return;
    }

    _decorations.initiateMove(_decorations.lastMove!);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      body: const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            Text('Welcome to the isabel flutter emebdder',
                style: TextStyle(color: Colors.black))
          ],
        ),
      ),
    );
  }
}
