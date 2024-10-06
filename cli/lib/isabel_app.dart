import 'package:flutter_tools/src/application_package.dart';
import 'package:flutter_tools/src/build_info.dart';
import 'package:flutter_tools/src/flutter_application_package.dart';
import 'package:flutter_tools/src/globals.dart' as globals;
import 'package:flutter_tools/src/base/file_system.dart';
import 'package:flutter_tools/src/project.dart';

import 'isabel_project.dart';

class IsabelApplicationPackageFactory extends FlutterApplicationPackageFactory {
  IsabelApplicationPackageFactory()
      : super(
          androidSdk: globals.androidSdk,
          processManager: globals.processManager,
          logger: globals.logger,
          userMessages: globals.userMessages,
          fileSystem: globals.fs,
        );

  @override
  Future<ApplicationPackage?> getPackageForPlatform(
    TargetPlatform platform, {
    BuildInfo? buildInfo,
    File? applicationBinary,
  }) async {
    if (platform == TargetPlatform.linux_x64) {
      return IsabelApp.fromProject(FlutterProject.current());
    }
    return super.getPackageForPlatform(platform,
        buildInfo: buildInfo, applicationBinary: applicationBinary);
  }
}

abstract class IsabelApp extends ApplicationPackage {
  IsabelApp({required String projectBundleId}) : super(id: projectBundleId);

  factory IsabelApp.fromProject(FlutterProject project) {
    return BuildableIsabelApp(project: IsabelProject.fromFlutter(project));
  }

  @override
  String get displayName => id;

  String executable(BuildMode buildMode, String targetArch);

  String outputDirectory(BuildMode buildMode, String targetArch);
}

class BuildableIsabelApp extends IsabelApp {
  BuildableIsabelApp({required this.project})
      : super(projectBundleId: project.parent.manifest.appName);

  final IsabelProject project;

  @override
  String executable(BuildMode buildMode, String targetArch) {
    return '';
  }

  @override
  String outputDirectory(BuildMode buildMode, String targetArch) {
    return '';
  }

  @override
  String get name => project.parent.manifest.appName;
}
