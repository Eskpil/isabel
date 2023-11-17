#!/bin/sh
set -ex

cd demo
flutter build bundle --local-engine=host_debug_unopt --local-engine-src-path=../../flutter-engine/src/
cd ..

cd build
ninja

./window-example

