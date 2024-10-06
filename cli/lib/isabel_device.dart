import 'dart:async';
import 'dart:io';

import 'package:flutter_tools/src/device.dart';
import 'package:flutter_tools/src/build_info.dart';
import 'package:flutter_tools/src/convert.dart';
import 'package:flutter_tools/src/project.dart';
import 'package:flutter_tools/src/device_port_forwarder.dart';

import './isabel_app.dart';

class IsabelDevice extends Device {
  IsabelDevice(super.id)
      : super(
            category: Category.desktop,
            platformType: PlatformType.custom,
            ephemeral: true) {}

  final IsabelLogReader _logReader = IsabelLogReader();

  @override
  final DevicePortForwarder portForwarder = const NoOpDevicePortForwarder();

  @override
  Future<bool> get isLocalEmulator async => false;

  @override
  Future<String?> get emulatorId async => null;

  @override
  Future<TargetPlatform> get targetPlatform async {
    // Use tester as a platform identifer for Isabel.
    // There's currently no other choice because getNameForTargetPlatform()
    // throws an error for unknown platform types.
    return TargetPlatform.linux_x64;
  }

  @override
  String get name => "Isabel";

  @override
  Future<String> get sdkNameAndVersion async => "linux-isabel-0.0.1-alpha";

  @override
  Future<bool> installApp(covariant IsabelApp app, {String? userIdentifier}) {
    // TODO: HACK
    return Future.value(false);
  }

  @override
  Future<bool> uninstallApp(covariant IsabelApp app, {String? userIdentifier}) {
    // TODO: HACK
    return Future.value(false);
  }

  @override
  Future<LaunchResult> startApp(
    IsabelApp package, {
    String? mainPath,
    String? route,
    DebuggingOptions? debuggingOptions,
    Map<String, Object?> platformArgs = const <String, Object>{},
    bool prebuiltApplication = false,
    bool ipv6 = false,
    String? userIdentifier,
  }) async {
    print('startApp');

    final BuildMode buildMode = debuggingOptions!.buildInfo.mode;
    package.executable();

    return LaunchResult.failed();
  }

  @override
  Future<bool> stopApp(covariant IsabelApp? app,
      {String? userIdentifier}) async {
    print("stopApp");
    return false;
  }

  @override
  void clearLogs() {}

  @override
  Future<void> dispose() {
    return Future.value();
  }

  @override
  FutureOr<DeviceLogReader> getLogReader({
    covariant IsabelApp? app,
    bool includePastLogs = false,
  }) =>
      _logReader;

  @override
  Future<bool> isAppInstalled(covariant IsabelApp app,
      {String? userIdentifier}) async {
    return false;
  }

  @override
  Future<bool> isLatestBuildInstalled(covariant IsabelApp app) async {
    return false;
  }

  bool isSupported() => true;

  @override
  bool isSupportedForProject(FlutterProject flutterProject) {
    return true;
  }
}

class IsabelLogReader extends DeviceLogReader {
  final StreamController<List<int>> _inputController =
      StreamController<List<int>>.broadcast();

  void initializeProcess(Process process) {
    process.stdout.listen(_inputController.add);
    process.stderr.listen(_inputController.add);
    process.exitCode.whenComplete(_inputController.close);
  }

  @override
  Stream<String> get logLines {
    return _inputController.stream
        .transform(utf8.decoder)
        .transform(const LineSplitter());
  }

  @override
  String get name => 'Isabel';

  @override
  void dispose() {}
}
