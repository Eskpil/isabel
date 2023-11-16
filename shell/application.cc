#include <cstring>
#include <cerrno>

#include "shell/application.h"
#include "shell/log.h"

namespace shell {

const struct aylin_application_listener Application::k_application_listener = {
    .output = [](struct aylin_application *aylin_app, struct aylin_output *output, void *userdata) {
      log::info("received output: {} ({}@{})", output->name, output->width, output->height);

    },
};

Application::Application() {
  // TODO: Allow user to tell us app-id.
  m_application = aylin_application_create("", &Application::k_application_listener, this);
  if (m_application == nullptr) {
    log::fatal("could not create aylin application: {}", strerror(errno));
  }

}

Application::~Application() {
  aylin_application_destroy(m_application);
}

void Application::attach(shell::Target *target) {
  // Target never outlives application anyway. And why would target try to
  // destroy us anyway?
  target->configure(this);
  m_targets.push_back(target);
}

int Application::exec() {
  for (auto &target : m_targets) {
    target->run();
  }

  return aylin_application_poll(m_application);
}

}