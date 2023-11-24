import 'package:demo/decorations.dart';
import 'package:flutter/material.dart';

import '../popups.dart';

class MainScreen extends StatelessWidget {
  const MainScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const MyHomePage(title: 'Demo page'),
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
  final Popups _popups = Popups();

  void _move(DragStartDetails details) {
    if (_decorations.last_move == null) {
      return;
    }

    _decorations.initiateMove(_decorations.last_move!);
  }

  void _spawnPopup(TapUpDetails details) {
    var x = details.globalPosition.dx;
    var y = details.globalPosition.dy;
    _popups.spawn("spawn_popup", x, y);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Theme.of(context).colorScheme.background,
      body: Center(
        child: Column(
//          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            GestureDetector(
              onPanStart: _move,
              child: Container(
                color: Theme.of(context).colorScheme.inversePrimary,
                width: 720,
                height: 32,
              ),
            ),
            GestureDetector(
              onSecondaryTapUp: _spawnPopup,
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Container(
                    width: 720,
                    height: 40,
                    color: Theme.of(context).colorScheme.primaryContainer,
                    child: const Text('hello'),
                  ),
                ],
              ),
            )
          ],
        ),
      ),
    );
  }
}
