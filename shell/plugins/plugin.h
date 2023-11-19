
#ifndef ISABEL_SHELL_PLUGIN_H
#define ISABEL_SHELL_PLUGIN_H

#include <memory>
#include <utility>

#include "shell/dispatcher.h"

#include <rapidjson/document.h>

namespace shell::plugins {

class MethodArguments {
public:
  virtual void serialize(rapidjson::Document &) = 0;
};

class MethodCall {
public:
  MethodCall(const char *message, size_t length) {
    m_document.Parse(message, length);
  }

  auto method() { return std::string(m_document["method"].GetString()); }

  template <typename T, typename... Args>
  std::unique_ptr<T> args(Args &&...args) {
    auto object = std::make_unique<T>(std::forward<Args>(args)...);
    object->serialize(m_document);
    return object;
  }

private:
  rapidjson::Document m_document;
};

class Plugin {
public:
  explicit Plugin(std::shared_ptr<Dispatcher> dispatcher)
      : m_dispatcher(std::move(dispatcher)) {}
  virtual ~Plugin() = default;

  virtual std::string_view name() = 0;

  std::shared_ptr<Dispatcher> dispatcher() { return m_dispatcher; }

private:
  std::shared_ptr<Dispatcher> m_dispatcher;
};
} // namespace shell::plugins

#endif // ISABEL_SHELL_PLUGIN_H
