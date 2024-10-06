import 'package:flutter_tools/src/base/os.dart';
import 'package:flutter_tools/src/doctor_validator.dart';
import 'package:flutter_tools/src/base/context.dart';

IsabelWorkflow? get isabelWorkflow => context.get<IsabelWorkflow>();

class IsabelWorkflow extends Workflow {
  IsabelWorkflow({
    required OperatingSystemUtils operatingSystemUtils,
  }) : _operatingSystemUtils = operatingSystemUtils;

  final OperatingSystemUtils _operatingSystemUtils;

  @override
  bool get appliesToHostPlatform =>
      (_operatingSystemUtils.hostPlatform == HostPlatform.linux_x64) ||
      (_operatingSystemUtils.hostPlatform == HostPlatform.linux_arm64);

  @override
  bool get canLaunchDevices => true;

  @override
  bool get canListDevices => true;

  @override
  bool get canListEmulators => false;
}
