import 'package:flutter_tools/src/base/analyze_size.dart';
import 'package:flutter_tools/src/base/os.dart';
import 'package:flutter_tools/src/build_info.dart';
import 'package:flutter_tools/src/commands/build.dart';
import 'package:flutter_tools/src/globals.dart' as globals;
import 'package:flutter_tools/src/project.dart';
import 'package:flutter_tools/src/runner/flutter_command.dart';

import '../isabel_builder.dart';

class IsabelBuildCommand extends BuildCommand {
  IsabelBuildCommand({bool verboseHelp = false})
      : super(
          artifacts: globals.artifacts!,
          fileSystem: globals.fs,
          buildSystem: globals.buildSystem,
          osUtils: globals.os,
          processUtils: globals.processUtils,
          verboseHelp: verboseHelp,
          androidSdk: globals.androidSdk,
          logger: globals.logger,
        ) {
    addSubcommand(BuildPackageCommand(verboseHelp: verboseHelp));
  }
}

class BuildPackageCommand extends BuildSubCommand {
  BuildPackageCommand({bool verboseHelp = false})
      : super(
          verboseHelp: verboseHelp,
          logger: globals.logger,
        ) {
    addCommonDesktopBuildOptions(verboseHelp: verboseHelp);
  }

  @override
  final String name = 'isabel';

  @override
  Future<Set<DevelopmentArtifact>> get requiredArtifacts async =>
      <DevelopmentArtifact>{
        DevelopmentArtifact.linux,
      };

  @override
  final String description = 'Build an eLinux package from your app.';

  @override
  Future<FlutterCommandResult> runCommand() async {
    // Not supported cross-building for x64 on arm64.
    final String? targetArch = 'x86-64';
    final String hostArch = _getCurrentHostPlatformArchName();
    if (hostArch != targetArch && hostArch == 'arm64') {
      globals.logger
          .printError('Not supported cross-building for x64 on arm64.');
      return FlutterCommandResult.fail();
    }

    final BuildInfo buildInfo = await getBuildInfo();
    final IsabelBuildInfo isabelBuildInfo = IsabelBuildInfo(
      buildInfo,
      targetArch: targetArch!,
      targetCompilerTriple: '',
    );
    //validateBuild(isabelBuildInfo);
    displayNullSafetyMode(buildInfo);

    await IsabelBuilder.buildBundle(
      project: FlutterProject.current(),
      targetFile: targetFile,
      isabelBuildInfo: isabelBuildInfo,
      sizeAnalyzer: SizeAnalyzer(
        analytics: globals.analytics,
        fileSystem: globals.fs,
        logger: globals.logger,
      ),
    );
    return FlutterCommandResult.success();
  }

  String _getCurrentHostPlatformArchName() {
    final HostPlatform hostPlatform = getCurrentHostPlatform();
    return hostPlatform.platformName;
  }
}
