import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

@pragma('vm:entry-point')
void isabelSidecarDemo() {
  print("running isabel sidecar demo");
  runApp(Container(color: Colors.black));
}

class SidecarRequestCreate {
  final double x, y;
  final int width, height;
  final String name;

  const SidecarRequestCreate(
      {required this.x,
      required this.y,
      required this.name,
      required this.width,
      required this.height});

  Map<dynamic, dynamic> serialize() {
    var data = {
      'anchorX': x.toInt(),
      'anchorY': y.toInt(),
      'anchorWidth': width,
      'anchorHeight': height,
      'name': name
    };
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

  void spawn(double x, double y, String name) {
    final req =
        SidecarRequestCreate(height: 8, width: 8, x: x, y: y, name: name);
    platform.invokeMethod('request_popup', req.serialize());
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
      home: const Layer(),
    );
  }
}

class Layer extends StatefulWidget {
  const Layer({super.key});

  @override
  State<Layer> createState() => _LayerState();
}

class _LayerState extends State<Layer> {
  @override
  Widget build(BuildContext context) {
    return Container(
        padding: const EdgeInsets.only(right: 720, left: 720),
        color: Theme.of(context).colorScheme.inversePrimary,
        child: const Row(children: [
          Text('Panel', style: TextStyle(fontSize: 12)),
          Spacer(),
          SidecarSpawner(
              name: "isabel.io/sidecar/demo",
              child: Text('Open popup', style: TextStyle(fontSize: 12))),
        ]));
  }
}

class SidecarSpawner extends StatefulWidget {
  final Widget child;
  final String name;
  const SidecarSpawner({super.key, required this.child, required this.name});

  @override
  State<SidecarSpawner> createState() => _SidecarSpawnerState();
}

class _SidecarSpawnerState extends State<SidecarSpawner> {
  final Sidecar _sidecar = Sidecar();

  void _onPressed(BuildContext context, TapDownDetails details) {
    final obj = context.findRenderObject()! as RenderBox;
    final rect = obj.paintBounds;

    final global = obj.localToGlobal(Offset.zero);

    final width = rect.width;

    final globalX = global.dx;
    final globalY = global.dy;

    final x = globalX + width / 2;
    final y = globalY - 17;

    _sidecar.spawn(x, y, widget.name);
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTapDown: (details) {
        _onPressed(context, details);
      },
      child: widget.child,
    );
  }
}
