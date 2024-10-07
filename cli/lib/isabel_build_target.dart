import 'package:flutter_tools/src/artifacts.dart';
import 'package:flutter_tools/src/base/common.dart';
import 'package:flutter_tools/src/base/file_system.dart';
import 'package:flutter_tools/src/base/process.dart';
import 'package:flutter_tools/src/build_info.dart';
import 'package:flutter_tools/src/build_system/build_system.dart';
import 'package:flutter_tools/src/build_system/depfile.dart';
import 'package:flutter_tools/src/build_system/exceptions.dart';
import 'package:flutter_tools/src/build_system/targets/android.dart';
import 'package:flutter_tools/src/build_system/targets/assets.dart';
import 'package:flutter_tools/src/build_system/targets/common.dart';
import 'package:flutter_tools/src/build_system/targets/icon_tree_shaker.dart';
import 'package:flutter_tools/src/globals.dart' as globals;
import 'package:flutter_tools/src/project.dart';

import 'isabel_builder.dart';
import 'isabel_project.dart';

abstract class IsabelAssetBundle extends Target {
  const IsabelAssetBundle(this.buildInfo);

  final IsabelBuildInfo buildInfo;

  @override
  String get name => 'elinux_asset_bundle';

  @override
  List<Source> get inputs => const <Source>[
        Source.pattern('{BUILD_DIR}/app.dill'),
        ...IconTreeShaker.inputs,
      ];

  @override
  List<Source> get outputs => const <Source>[];

  @override
  List<String> get depfiles => <String>[
        'flutter_assets.d',
      ];

  @override
  List<Target> get dependencies => const <Target>[
        KernelSnapshot(),
      ];

  @override
  Future<void> build(Environment environment) async {
    if (environment.defines[kBuildMode] == null) {
      throw MissingDefineException(kBuildMode, name);
    }
    final BuildMode buildMode =
        BuildMode.fromCliName(environment.defines[kBuildMode]!);
    final Directory outputDirectory = environment.outputDir
        .childDirectory('flutter_assets')
      ..createSync(recursive: true);

    // Only copy the prebuilt runtimes and kernel blob in debug mode.
    if (buildMode == BuildMode.debug) {
      final String vmSnapshotData = environment.artifacts
          .getArtifactPath(Artifact.vmSnapshotData, mode: BuildMode.debug);
      final String isolateSnapshotData = environment.artifacts
          .getArtifactPath(Artifact.isolateSnapshotData, mode: BuildMode.debug);
      environment.buildDir
          .childFile('app.dill')
          .copySync(outputDirectory.childFile('kernel_blob.bin').path);
      environment.fileSystem
          .file(vmSnapshotData)
          .copySync(outputDirectory.childFile('vm_snapshot_data').path);
      environment.fileSystem
          .file(isolateSnapshotData)
          .copySync(outputDirectory.childFile('isolate_snapshot_data').path);
    }
    final TargetPlatform tp = buildInfo.targetArch == 'arm64'
        ? TargetPlatform.linux_arm64
        : TargetPlatform.linux_x64;
    final Depfile assetDepfile = await copyAssets(
      environment,
      outputDirectory,
      targetPlatform: tp,
      buildMode: buildMode,
      flavor: environment.defines[kFlavor],
    );
    final DepfileService depfileService = DepfileService(
      fileSystem: environment.fileSystem,
      logger: environment.logger,
    );
    depfileService.writeToFile(
      assetDepfile,
      environment.buildDir.childFile('flutter_assets.d'),
    );
  }
}

class ReleaseIsabelApplication extends IsabelAssetBundle {
  ReleaseIsabelApplication(super.buildInfo);

  @override
  String get name => 'release_elinux_application';

  @override
  List<Target> get dependencies => <Target>[
        ...super.dependencies,
        IsabelAotElf(buildInfo.targetArch == 'arm64'
            ? TargetPlatform.linux_arm64
            : TargetPlatform.linux_x64),
      ];
}

class IsabelAotElf extends AotElfBase {
  const IsabelAotElf(this.targetPlatform);

  @override
  String get name => 'elinux_aot_elf';

  @override
  List<Source> get inputs => <Source>[
        const Source.pattern(
            '{FLUTTER_ROOT}/packages/flutter_tools/lib/src/build_system/targets/common.dart'),
        const Source.pattern('{BUILD_DIR}/app.dill'),
        const Source.artifact(Artifact.engineDartBinary),
        const Source.artifact(Artifact.skyEnginePath),
        Source.artifact(
          Artifact.genSnapshot,
          platform: targetPlatform,
          mode: BuildMode.release,
        ),
      ];

  @override
  List<Source> get outputs => const <Source>[
        Source.pattern('{BUILD_DIR}/app.so'),
      ];

  @override
  List<Target> get dependencies => const <Target>[
        KernelSnapshot(),
      ];

  final TargetPlatform targetPlatform;
}

/// Source: [DebugAndroidApplication] in `android.dart`
class DebugIsabelApplication extends IsabelAssetBundle {
  DebugIsabelApplication(super.buildInfo);

  @override
  String get name => 'debug_isabel_application';

  @override
  List<Source> get inputs => <Source>[
        ...super.inputs,
        const Source.artifact(Artifact.vmSnapshotData, mode: BuildMode.debug),
        const Source.artifact(Artifact.isolateSnapshotData,
            mode: BuildMode.debug),
      ];

  @override
  List<Source> get outputs => <Source>[
        ...super.outputs,
        const Source.pattern('{OUTPUT_DIR}/flutter_assets/vm_snapshot_data'),
        const Source.pattern(
            '{OUTPUT_DIR}/flutter_assets/isolate_snapshot_data'),
        const Source.pattern('{OUTPUT_DIR}/flutter_assets/kernel_blob.bin'),
      ];

  @override
  List<Target> get dependencies => <Target>[
        ...super.dependencies,
      ];
}

class NativeBundle {
  NativeBundle(this.buildInfo, this.targetFile);

  final IsabelBuildInfo? buildInfo;
  final String? targetFile;

  final ProcessUtils _processUtils = ProcessUtils(
      logger: globals.logger, processManager: globals.processManager);

  Future<void> build(Environment environment) async {
    final FlutterProject project =
        FlutterProject.fromDirectory(environment.projectDir);
    final IsabelProject isabelProject = IsabelProject.fromFlutter(project);

    // Clean up the intermediate and output directories.
    final Directory isabelDir = isabelProject.editableDirectory;
    final Directory outputDir =
        environment.outputDir.childDirectory(buildInfo!.targetArch);
    if (outputDir.existsSync()) {
      outputDir.deleteSync(recursive: true);
    }
    outputDir.createSync(recursive: true);

    final Directory outputBundleDir = outputDir.childDirectory('bundle');
    if (outputBundleDir.existsSync()) {
      outputBundleDir.deleteSync(recursive: true);
    }
    outputBundleDir.createSync(recursive: true);

    final Directory outputBundleLibDir = outputBundleDir.childDirectory('lib');
    if (outputBundleLibDir.existsSync()) {
      outputBundleLibDir.deleteSync(recursive: true);
    }
    outputBundleLibDir.createSync(recursive: true);

    final Directory outputBundleDataDir =
        outputBundleDir.childDirectory('data');
    if (outputBundleDataDir.existsSync()) {
      outputBundleDataDir.deleteSync(recursive: true);
    }
    outputBundleDataDir.createSync(recursive: true);

    // Copy necessary files
    final Directory engineDir = globals.cache.getArtifactDirectory('engine');
    final Directory flutterDir = isabelDir.childDirectory('flutter');
    final Directory commonDir =
        engineDir.parent.childDirectory('engine/linux-x64');
    final Directory flutterEphemeralDir =
        flutterDir.childDirectory('ephemeral');
    // Copy necessary files.
    {
      flutterEphemeralDir.createSync(recursive: true);

      commonDir.listSync().whereType<File>().forEach((File lib) =>
          lib.copySync(flutterEphemeralDir.childFile(lib.basename).path));

      final File icuData = commonDir.childFile('icudtl.dat');
      icuData.copySync(flutterEphemeralDir.childFile(icuData.basename).path);

      if (buildInfo!.buildInfo.mode.isPrecompiled) {
        final File aotSharedLib = environment.buildDir.childFile('app.so');
        aotSharedLib.copySync(outputBundleLibDir.childFile('libapp.so').path);
      }
    }

    final List<String> cmd = ['cargo', 'build'];

    if (buildInfo!.buildInfo.isRelease) {
      cmd.add('--release');
    }

    // Run the native build.
    RunResult result = await _processUtils.run(
      cmd,
      workingDirectory: isabelDir.path,
      environment: <String, String>{
        'CARGO_TARGET_DIR': outputDir.path,
      },
    );

    if (result.exitCode != 0) {
      throwToolExit('Failed to cargo:\n$result');
    }

    {
      final Directory flutterAssetsDir =
          outputBundleDataDir.childDirectory('flutter_assets');
      copyDirectory(
        environment.outputDir.childDirectory('flutter_assets'),
        flutterAssetsDir,
      );
    }
  }
}
