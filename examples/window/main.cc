#include <memory>

#include "shell/application.h"
#include "shell/window.h"

int main() {
  auto application = std::make_unique<shell::Application>();

  auto window = std::make_unique<shell::Window>(shell::WindowOptions{
      .width = 1280,
      .height = 720,
      .title = "Example",
  });
  application->attach(window.release());

  return application->exec();
}