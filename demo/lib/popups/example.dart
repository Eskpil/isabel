import 'package:flutter/material.dart';

class ExamplePopup extends StatelessWidget {
  const ExamplePopup({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const Popup(),
    );
  }
}

class Popup extends StatefulWidget {
  const Popup({super.key});

  @override
  State<Popup> createState() => _PopupState();
}

class _PopupState extends State<Popup> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Theme.of(context).colorScheme.background,
      body: const Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [Text('hello popup')],
      ),
    );
  }
}
