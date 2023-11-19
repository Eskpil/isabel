#ifndef ISABEL_SHELL_PLUGINS_TEXT_INPUT_ARGS_H
#define ISABEL_SHELL_PLUGINS_TEXT_INPUT_ARGS_H

#include "shell/plugins/plugin.h"

namespace shell::plugins {
class SetClientArgs : public MethodArguments {
public:
  void serialize(rapidjson::Document &document) {
    // args is array with two elements. First element is integer (client_id) and
    // second argument is object containing all our valuable information.
    auto args = document["args"].GetArray()[1].GetObject();

    client_id = document["args"].GetArray()[0].GetInt();

    input_type = args["inputType"].GetObject()["name"].GetString();
    read_only = args["readOnly"].GetBool();
    obscure_text = args["obscureText"].GetBool();
    autocorrect = args["autocorrect"].GetBool();
    smart_dashes_type = args["smartDashesType"].GetString();
    smart_quotes_type = args["smartQuotesType"].GetString();
    enable_interactive_selection = args["enableInteractiveSelection"].GetBool();
    input_action = args["inputAction"].GetString();
    text_capitalization = args["textCapitalization"].GetString();
    keyboard_appearance = args["keyboardAppearance"].GetString();
    enable_ime_personalized_learning =
        args["enableIMEPersonalizedLearning"].GetBool();

    // TODO: Handle contentCommitMimeTypes
    enable_delta_model = args["enableDeltaModel"].GetBool();
  }

  int client_id;

  std::string_view input_type;
  bool read_only;
  bool obscure_text;
  bool autocorrect;
  std::string_view smart_dashes_type;
  std::string_view smart_quotes_type;
  bool enable_suggestions;
  bool enable_interactive_selection;
  std::string_view input_action;
  std::string_view text_capitalization;
  std::string_view keyboard_appearance;
  bool enable_ime_personalized_learning;
  std::vector<std::string_view> content_commit_mime_types;
  bool enable_delta_model;
};

} // namespace shell::plugins

#endif // ISABEL_SHELL_PLUGINS_TEXT_INPUT_ARGS_H