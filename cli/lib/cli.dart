import 'dart:io';

import 'package:flutter_tools/executable.dart' as flutter;
import 'package:flutter_tools/runner.dart' as runner;
import 'package:flutter_tools/src/application_package.dart';
import 'package:flutter_tools/src/artifacts.dart';
import 'package:flutter_tools/src/base/context.dart';
import 'package:flutter_tools/src/base/io.dart';
import 'package:flutter_tools/src/base/logger.dart';
import 'package:flutter_tools/src/base/template.dart';
import 'package:flutter_tools/src/build_system/build_targets.dart';
import 'package:flutter_tools/src/cache.dart';
import 'package:flutter_tools/src/commands/config.dart';
import 'package:flutter_tools/src/commands/devices.dart';
import 'package:flutter_tools/src/commands/doctor.dart';
import 'package:flutter_tools/src/commands/emulators.dart';
import 'package:flutter_tools/src/commands/generate_localizations.dart';
import 'package:flutter_tools/src/commands/install.dart';
import 'package:flutter_tools/src/commands/logs.dart';
import 'package:flutter_tools/src/commands/screenshot.dart';
import 'package:flutter_tools/src/commands/symbolize.dart';
import 'package:flutter_tools/src/device.dart';
import 'package:flutter_tools/src/doctor.dart';
import 'package:flutter_tools/src/features.dart';
import 'package:flutter_tools/src/globals.dart' as globals;
import 'package:flutter_tools/src/isolated/build_targets.dart';
import 'package:flutter_tools/src/isolated/mustache_template.dart';
import 'package:flutter_tools/src/runner/flutter_command.dart';
import 'package:path/path.dart';

import 'commands/run.dart';
import 'commands/build.dart';
import 'isabel_device_discovery.dart';
import 'isabel_workflow.dart';
import 'isabel_app.dart';

Future<void> main(List<String> args) async {
  final bool veryVerbose = args.contains('-vv');
  final bool verbose =
      args.contains('-v') || args.contains('--verbose') || veryVerbose;

  final bool doctor = (args.isNotEmpty && args.first == 'doctor') ||
      (args.length == 2 && verbose && args.last == 'doctor');
  final bool help = args.contains('-h') ||
      args.contains('--help') ||
      (args.isNotEmpty && args.first == 'help') ||
      (args.length == 1 && verbose);
  final bool muteCommandLogging = (help || doctor) && !veryVerbose;
  final bool verboseHelp = help && verbose;

  final bool hasSpecifiedDeviceId =
      args.contains('-d') || args.contains('--device-id');

  args = <String>[
    '--suppress-analytics', // Suppress flutter analytics by default.
    '--no-version-check',
    if (!hasSpecifiedDeviceId) ...<String>['--device-id', 'isabel'],
    ...args,
  ];

  //args.add("run");

  Cache.flutterRoot = '/opt/flutter';

  await runner.run(
      args,
      () => <FlutterCommand>[
            // Commands directly from flutter_tools.
            ConfigCommand(verboseHelp: verboseHelp),
            DevicesCommand(verboseHelp: verboseHelp),
            DoctorCommand(verbose: verbose),
            EmulatorsCommand(),
            GenerateLocalizationsCommand(
              fileSystem: globals.fs,
              logger: globals.logger,
              artifacts: globals.artifacts!,
              processManager: globals.processManager,
            ),
            InstallCommand(verboseHelp: verboseHelp),
            LogsCommand(
              sigint: ProcessSignal.sigint,
              sigterm: ProcessSignal.sigterm,
            ),
            ScreenshotCommand(fs: globals.fs),
            SymbolizeCommand(stdio: globals.stdio, fileSystem: globals.fs),

            IsabelRunCommand(verboseHelp: verboseHelp),
            IsabelBuildCommand(verboseHelp: verboseHelp),
          ],
      overrides: <Type, Generator>{
        ApplicationPackageFactory: () => IsabelApplicationPackageFactory(),
        BuildTargets: () => const BuildTargetsImpl(),
        DeviceManager: () => IsabelDeviceManager(),
        IsabelWorkflow: () => IsabelWorkflow(operatingSystemUtils: globals.os),
      },
      verbose: verbose,
      verboseHelp: verboseHelp,
      muteCommandLogging: muteCommandLogging,
      reportCrashes: false,
      shutdownHooks: globals.shutdownHooks);
}

String get rootPath {
  final String scriptPath = Platform.script.toFilePath();
  return normalize(join(
    scriptPath,
    scriptPath.endsWith('.snapshot') ? '../../..' : '../..',
  ));
}
