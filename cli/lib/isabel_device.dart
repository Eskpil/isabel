import 'dart:async';
import 'dart:io';

import 'package:flutter_tools/src/device.dart';
import 'package:flutter_tools/src/build_info.dart';
import 'package:flutter_tools/src/convert.dart';
import 'package:flutter_tools/src/project.dart';
import 'package:flutter_tools/src/device_port_forwarder.dart';
import 'package:flutter_tools/src/protocol_discovery.dart';
import 'package:flutter_tools/src/base/logger.dart';
import 'package:process/process.dart';

import './isabel_app.dart';
import './isabel_builder.dart';

class IsabelDevice extends Device {
  IsabelDevice(super.id,
      {required this.targetArch,
      required this.processManager,
      required this.logger})
      : super(
            category: Category.desktop,
            platformType: PlatformType.custom,
            ephemeral: true);

  final String targetArch;
  final ProcessManager processManager;
  final Set<Process> runningProcesses = <Process>{};
  final Logger logger;

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

    await buildForDevice(
      package,
      buildInfo: debuggingOptions!.buildInfo,
      mainPath: mainPath,
    );

    if (package is! BuildableIsabelApp) {
      return LaunchResult.failed();
    }

    final BuildMode buildMode = debuggingOptions.buildInfo.mode;
    final String executable = package.executable(buildMode, targetArch);
    final String bundle = package.outputDirectory(buildMode, targetArch);

    final String icudtl = package.project.editableDirectory
        .childDirectory('flutter')
        .childDirectory('ephemeral')
        .childFile('icudtl.dat')
        .path;

    final String executableOptions = '--assets $bundle --icudtl $icudtl';

    print(executableOptions);

    final Process process = await processManager.start(
      <String>[
        executable,
        '--assets=$bundle',
        '--icudtl=$icudtl',
        ...debuggingOptions.dartEntrypointArgs,
      ],
      environment: _computeEnvironment(debuggingOptions, false, route),
    );

    runningProcesses.add(process);
    unawaited(process.exitCode.then((_) => runningProcesses.remove(process)));

    _logReader.initializeProcess(process);
    if (debuggingOptions.buildInfo.isRelease == true) {
      return LaunchResult.succeeded();
    }

    final ProtocolDiscovery observatoryDiscovery = ProtocolDiscovery.vmService(
      _logReader,
      devicePort: debuggingOptions.deviceVmServicePort,
      hostPort: debuggingOptions.hostVmServicePort,
      ipv6: ipv6,
      logger: logger,
    );
    try {
      final Uri? observatoryUri = await observatoryDiscovery.uri;
      if (observatoryUri != null) {
        return LaunchResult.succeeded(observatoryUri: observatoryUri);
      }
      logger.printError(
        'Error waiting for a debug connection: '
        'The log reader stopped unexpectedly.',
      );
    } on Exception catch (error) {
      logger.printError('Error waiting for a debug connection: $error');
    } finally {
      await observatoryDiscovery.cancel();
    }
    return LaunchResult.failed();
  }

  Future<void> buildForDevice(
    IsabelApp package, {
    String? mainPath,
    BuildInfo? buildInfo,
  }) async {
    final FlutterProject project = FlutterProject.current();
    final IsabelBuildInfo isabelBuildInfo = IsabelBuildInfo(
      buildInfo!,
      targetArch: targetArch,
      targetCompilerTriple: null,
    );
    await IsabelBuilder.buildBundle(
      project: project,
      targetFile: mainPath!,
      isabelBuildInfo: isabelBuildInfo,
    );
    package = IsabelApp.fromProject(project);
  }

  Map<String, String> _computeEnvironment(
      DebuggingOptions debuggingOptions, bool traceStartup, String? route) {
    int flags = 0;
    final Map<String, String> environment = <String, String>{};

    void addFlag(String value) {
      flags += 1;
      environment['FLUTTER_ENGINE_SWITCH_$flags'] = value;
    }

    void finish() {
      environment['FLUTTER_ENGINE_SWITCHES'] = flags.toString();
    }

    addFlag('enable-dart-profiling=true');

    if (traceStartup) {
      addFlag('trace-startup=true');
    }
    if (route != null) {
      addFlag('route=$route');
    }
    if (debuggingOptions.enableSoftwareRendering) {
      addFlag('enable-software-rendering=true');
    }
    if (debuggingOptions.skiaDeterministicRendering) {
      addFlag('skia-deterministic-rendering=true');
    }
    if (debuggingOptions.traceSkia) {
      addFlag('trace-skia=true');
    }
    if (debuggingOptions.traceAllowlist != null) {
      addFlag('trace-allowlist=${debuggingOptions.traceAllowlist}');
    }
    if (debuggingOptions.traceSkiaAllowlist != null) {
      addFlag('trace-skia-allowlist=${debuggingOptions.traceSkiaAllowlist}');
    }
    if (debuggingOptions.traceSystrace) {
      addFlag('trace-systrace=true');
    }
    if (debuggingOptions.endlessTraceBuffer) {
      addFlag('endless-trace-buffer=true');
    }
    if (debuggingOptions.dumpSkpOnShaderCompilation) {
      addFlag('dump-skp-on-shader-compilation=true');
    }
    if (debuggingOptions.cacheSkSL) {
      addFlag('cache-sksl=true');
    }
    if (debuggingOptions.purgePersistentCache) {
      addFlag('purge-persistent-cache=true');
    }
    // Options only supported when there is a VM Service connection between the
    // tool and the device, usually in debug or profile mode.
    if (debuggingOptions.debuggingEnabled) {
      if (debuggingOptions.deviceVmServicePort != null) {
        addFlag('observatory-port=${debuggingOptions.deviceVmServicePort}');
      }
      if (debuggingOptions.buildInfo.isDebug) {
        addFlag('enable-checked-mode=true');
        addFlag('verify-entry-points=true');
      }
      if (debuggingOptions.startPaused) {
        addFlag('start-paused=true');
      }
      if (debuggingOptions.disableServiceAuthCodes) {
        addFlag('disable-service-auth-codes=true');
      }
      final String dartVmFlags = computeDartVmFlags(debuggingOptions);
      if (dartVmFlags.isNotEmpty) {
        addFlag('dart-flags=$dartVmFlags');
      }
      if (debuggingOptions.useTestFonts) {
        addFlag('use-test-fonts=true');
      }
      if (debuggingOptions.verboseSystemLogs) {
        addFlag('verbose-logging=true');
      }
    }
    finish();
    return environment;
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
