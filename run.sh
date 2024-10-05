#!/bin/sh

cd demo
flutter build bundle --local-engine=host_debug_unopt --local-engine-host=host_debug_unopt --local-engine-src-path=/home/linus/repos/google/src
cd ..
cargo r
