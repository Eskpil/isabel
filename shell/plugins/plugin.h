
#ifndef ISABEL_SHELL_PLUGIN_H
#define ISABEL_SHELL_PLUGIN_H

#include <cstring>
#include <memory>
#include <utility>

#include "shell/dispatcher.h"
#include "shell/log.h"

#include <json-c/json.h>

namespace shell::plugins {

class MethodArguments {
public:
  virtual void deserialize(struct json_object *){};
  virtual void serialize(struct json_object *){};
  virtual enum json_type type() { return json_type_array; };
};

class MethodCall {
public:
  explicit MethodCall(const char *message, size_t length) {
    char *n_message = static_cast<char *>(malloc(length + 1));
    memcpy(n_message, message, length);
    n_message[length] = '\0';

    m_object = json_tokener_parse(n_message);

    free(n_message);
  }

  explicit MethodCall(std::string_view method,
                      std::shared_ptr<MethodArguments> args)
      : m_args(std::move(args)), m_method(method) {
    m_object = json_object_new_object();

    auto method_object = json_object_new_string(m_method.data());
    json_object_object_add(m_object, "method", method_object);

    struct json_object *args_object = nullptr;
    // TODO: Hack, not all argument serializers want arrays.
    switch (m_args->type()) {
    case json_type_object:
      args_object = json_object_new_object();
      break;
    case json_type_array:
      args_object = json_object_new_array();
      break;
    default:
      log::fatal("can not handle json type: {}",
                 static_cast<int>(m_args->type()));
      break;
    }
    json_object_new_array();

    m_args->serialize(args_object);
    json_object_object_add(m_object, "args", args_object);
  }

  ~MethodCall() {
    json_object_put(m_object);
    m_object = nullptr;
  }

  std::string method() {
    return {json_object_get_string(json_object_object_get(m_object, "method"))};
  }

  template <typename T, typename... Args>
  std::unique_ptr<T> args(Args &&...args) {
    auto object = std::make_unique<T>(std::forward<Args>(args)...);
    struct json_object *arguments_obj =
        json_object_object_get(m_object, "args");
    object->deserialize(arguments_obj);
    return object;
  }

  const char *build() { return json_object_to_json_string(m_object); }

private:
  std::string m_method;
  std::shared_ptr<MethodArguments> m_args;

  struct json_object *m_object;
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
