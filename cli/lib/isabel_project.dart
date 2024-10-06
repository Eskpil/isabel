import 'package:flutter_tools/src/project.dart';
import 'package:flutter_tools/src/base/file_system.dart';

class IsabelProject extends FlutterProjectPlatform {
  IsabelProject.fromFlutter(this.parent);
  @override
  final FlutterProject parent;

  @override
  String get pluginConfigKey => 'isabel';

  String get _childDirectory => 'isabel';

  @override
  bool existsSync() => editableDirectory.existsSync();

  Directory get editableDirectory =>
      parent.directory.childDirectory(_childDirectory);
}
