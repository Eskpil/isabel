import 'package:flutter_tools/src/cache.dart';
import 'package:flutter_tools/src/commands/run.dart';

class IsabelRunCommand extends RunCommand {
  IsabelRunCommand({super.verboseHelp});

  @override
  Future<Set<DevelopmentArtifact>> get requiredArtifacts async =>
      <DevelopmentArtifact>{
        // TODO: This is probably hack
        DevelopmentArtifact.linux,
      };
}
