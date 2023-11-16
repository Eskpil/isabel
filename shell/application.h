#ifndef ISABEL_SHELL_APPLICATION_H_
#define ISABEL_SHELL_APPLICATION_H_

#include <vector>

#include <aylin/aylin.h>

#include "shell/target.h"

namespace shell {

class Application {
public:
  explicit Application();
  ~Application();

  // expects to be in control of the target after hand over.
  void attach(Target *);

  [[nodiscard]] struct aylin_application *inner() const { return m_application; }

  int exec();
private:
  static const struct aylin_application_listener k_application_listener;

  std::vector<Target*> m_targets;

  struct aylin_application *m_application;
};
}

#endif // ISABEL_SHELL_APPLICATION_H_