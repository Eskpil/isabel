import 'package:flutter_tools/src/flutter_device_manager.dart';
import 'package:flutter_tools/src/device.dart';
import 'package:flutter_tools/src/globals.dart' as globals;
import 'package:flutter_tools/src/macos/macos_workflow.dart';
import 'package:flutter_tools/src/windows/windows_workflow.dart';
import 'package:flutter_tools/src/fuchsia/fuchsia_workflow.dart';
import 'package:flutter_tools/src/base/context.dart';
import 'package:flutter_tools/src/features.dart';
import 'package:flutter_tools/src/android/android_workflow.dart';
import 'package:flutter_tools/src/custom_devices/custom_devices_config.dart';
import 'package:flutter_tools/src/base/logger.dart';
import 'package:process/process.dart';

import './isabel_workflow.dart';
import './isabel_device.dart';

class IsabelDeviceManager extends FlutterDeviceManager {
  IsabelDeviceManager()
      : super(
          logger: globals.logger,
          processManager: globals.processManager,
          platform: globals.platform,
          androidSdk: globals.androidSdk,
          iosSimulatorUtils: globals.iosSimulatorUtils!,
          featureFlags: featureFlags,
          fileSystem: globals.fs,
          iosWorkflow: globals.iosWorkflow!,
          artifacts: globals.artifacts!,
          flutterVersion: globals.flutterVersion,
          androidWorkflow: androidWorkflow!,
          fuchsiaWorkflow: fuchsiaWorkflow!,
          xcDevice: globals.xcdevice!,
          userMessages: globals.userMessages,
          windowsWorkflow: windowsWorkflow!,
          macOSWorkflow: context.get<MacOSWorkflow>()!,
          fuchsiaSdk: globals.fuchsiaSdk!,
          operatingSystemUtils: globals.os,
          customDevicesConfig: CustomDevicesConfig(
            fileSystem: globals.fs,
            logger: globals.logger,
            platform: globals.platform,
          ),
        );

  final IsabelDeviceDiscovery _isabelDeviceDiscovery = IsabelDeviceDiscovery(
    isabelWorkflow: isabelWorkflow!,
    logger: globals.logger,
    processManager: globals.processManager,
  );

  @override
  List<DeviceDiscovery> get deviceDiscoverers => <DeviceDiscovery>[
        ...super.deviceDiscoverers,
        _isabelDeviceDiscovery,
      ];
}

class IsabelDeviceDiscovery extends PollingDeviceDiscovery {
  IsabelDeviceDiscovery({
    required this.logger,
    required this.processManager,
    required this.isabelWorkflow,
  }) : super('Isabel Devices');

  final Logger logger;
  final ProcessManager processManager;
  final IsabelWorkflow isabelWorkflow;

  @override
  bool get supportsPlatform => isabelWorkflow.appliesToHostPlatform;

  @override
  bool get canListAnything => isabelWorkflow.canListDevices;

  @override
  Future<List<Device>> pollingGetDevices({Duration? timeout}) async {
    List<Device> devices = [];

    devices.add(IsabelDevice('isabel-desktop',
        targetArch: "x86-64", logger: logger, processManager: processManager));

    return devices;
  }

  @override
  Future<List<String>> getDiagnostics() async => const <String>[];

  @override
  List<String> get wellKnownIds => const <String>['isabel'];
}
