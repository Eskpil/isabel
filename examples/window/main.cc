#include <memory>

#include "shell/application.h"
#include "shell/window.h"

int main() {
  auto application = std::make_unique<shell::Application>();

  // Currently, the example is expected to be executed from the build/
  // directory.
  auto window = std::make_unique<shell::Window>(shell::WindowOptions{
      .width = 1280,
      .height = 720,
      .title = "Example 1",

      .icudtl_path = "../demo/icudtl.dat",
      .flutter_assets_path = "../demo/build/flutter_assets",
  });
  application->attach(window.release());

  return application->exec();
}