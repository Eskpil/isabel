#ifndef ISABEL_SHELL_PLUGINS_TEXT_INPUT_ARGS_H
#define ISABEL_SHELL_PLUGINS_TEXT_INPUT_ARGS_H

#include "shell/plugins/plugin.h"

namespace shell::plugins {
class SetClientArgs : public MethodArguments {
public:
  void deserialize(struct json_object *object) {
    // args is array with two elements. First element is integer (client_id) and
    // second argument is object containing all our valuable information.

    if (!json_object_is_type(object, json_type_array)) {
      log::fatal("object is not array");
    }

    // client_id
    {
      auto client_id_obj = json_object_array_get_idx(object, 0);
      client_id = json_object_get_int(client_id_obj);
    }

    auto args = json_object_array_get_idx(object, 1);

    // TODO: input_type = args["inputType"].GetObject()["name"].GetString();
    {
      auto read_only_obj = json_object_object_get(args, "readOnly");
      read_only = json_object_get_boolean(read_only_obj);
    }
    {
      auto obscure_text_obj = json_object_object_get(args, "obscureText");
      obscure_text = json_object_get_boolean(obscure_text_obj);
    }
    {
      auto autocorrect_obj = json_object_object_get(args, "autocorrect");
      autocorrect = json_object_get_boolean(autocorrect_obj);
    }
    {
      auto smart_dashes_type_obj =
          json_object_object_get(args, "smartDashesType");
      smart_dashes_type =
          std::string_view(json_object_get_string(smart_dashes_type_obj));
    }
    {
      auto smart_quotes_type_obj =
          json_object_object_get(args, "smartQuotesType");
      smart_quotes_type =
          std::string_view(json_object_get_string(smart_quotes_type_obj));
    }
    {
      auto enable_interactive_selection_obj =
          json_object_object_get(args, "enableInteractiveSelection");
      enable_interactive_selection =
          json_object_get_boolean(enable_interactive_selection_obj);
    }
    {
      auto input_action_obj = json_object_object_get(args, "inputAction");
      input_action = std::string_view(json_object_get_string(input_action_obj));
    }
    {
      auto text_capitalization_obj =
          json_object_object_get(args, "textCapitalization");
      text_capitalization =
          std::string_view(json_object_get_string(text_capitalization_obj));
    }
    {
      auto keyboard_appearance_obj =
          json_object_object_get(args, "keyboardAppearance");
      keyboard_appearance =
          std::string_view(json_object_get_string(keyboard_appearance_obj));
    }
    {
      auto enable_ime_personalized_learning_obj =
          json_object_object_get(args, "enableIMEPersonalizedLearning");
      enable_ime_personalized_learning =
          json_object_get_boolean(enable_ime_personalized_learning_obj);
    }
    {
      auto enable_delta_model_obj = json_object_object_get(args, "enableDelta");
      enable_delta_model = json_object_get_boolean(enable_delta_model_obj);
    }
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

class UpdateEditingStateArgs : public MethodArguments {
public:
  void serialize(struct json_object *object) {
    {
      auto client_id_obj = json_object_new_int(client_id);
      json_object_array_add(object, client_id_obj);
    }

    auto args_obj = json_object_new_object();

    {
      auto text_obj = json_object_new_string(text->c_str());
      json_object_object_add(args_obj, "text", text_obj);
    }
    {
      auto selection_base_obj = json_object_new_int(selection_base);
      json_object_object_add(args_obj, "selectionBase", selection_base_obj);
    }
    {
      auto selection_extent_obj = json_object_new_int(selection_extent);
      json_object_object_add(args_obj, "selectionExtent", selection_extent_obj);
    }
    {
      auto composing_base_obj = json_object_new_int(composing_base);
      json_object_object_add(args_obj, "composingBase", composing_base_obj);
    }
    {
      auto composing_extent_obj = json_object_new_int(composing_extent);
      json_object_object_add(args_obj, "composingExtent", composing_extent_obj);
    }
    {
      auto selection_affinity_obj =
          json_object_new_string(selection_affinity->c_str());
      json_object_object_add(args_obj, "selectionAffinity",
                             selection_affinity_obj);
    }
    {
      auto is_directional_obj = json_object_new_boolean(is_directional);
      json_object_object_add(args_obj, "isDirectional", is_directional_obj);
    }

    json_object_array_add(object, args_obj);
  }

  int client_id;
  int composing_base;
  int composing_extent;
  std::shared_ptr<std::string> selection_affinity;
  int selection_base;
  int selection_extent;
  bool is_directional;
  std::shared_ptr<std::string> text;
};

class SetEditingStateArgs : public MethodArguments {
public:
  void deserialize(struct json_object *object) {
    {
      auto text_obj = json_object_object_get(object, "text");
      text = std::string_view(json_object_get_string(text_obj));
    }
    {
      auto selection_base_obj = json_object_object_get(object, "selectionBase");
      selection_base = json_object_get_int(selection_base_obj);
    }
    {
      auto selection_extent_obj =
          json_object_object_get(object, "selectionExtent");
      selection_extent = json_object_get_int(selection_extent_obj);
    }
    {
      auto selection_affinity_obj =
          json_object_object_get(object, "selectionAffinity");
      selection_affinity =
          std::string_view(json_object_get_string(selection_affinity_obj));
    }
    {
      auto selection_is_directional_obj =
          json_object_object_get(object, "selectionIsDirectional");
      selection_is_directional =
          json_object_get_boolean(selection_is_directional_obj);
    }
  }

  std::string text;
  int selection_base;
  int selection_extent;
  std::string selection_affinity;
  bool selection_is_directional;
};

class PerformActionArgs : public MethodArguments {
public:
  void serialize(struct json_object *object) {
    {
      auto client_id_obj = json_object_new_int(client_id);
      json_object_array_add(object, client_id_obj);
    }
    {
      auto input_action_obj = json_object_new_string(input_action.data());
      json_object_array_add(object, input_action_obj);
    }
  }

  int client_id;
  std::string input_action;
};

} // namespace shell::plugins

#endif // ISABEL_SHELL_PLUGINS_TEXT_INPUT_ARGS_H