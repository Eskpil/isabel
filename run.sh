#!/bin/sh

pushd apps/networksettings
flutter build bundle --local-engine=host_debug_unopt --local-engine-host=host_debug_unopt --local-engine-src-path=/home/linus/repos/google/src

if [ $? -ne 0 ]; then
exit
fi

popd

if [ "$1" ]; then
  cargo r -- --build apps/networksettings/build
else
  target/debug/isabel-rs --build apps/networksettings/build
fi
