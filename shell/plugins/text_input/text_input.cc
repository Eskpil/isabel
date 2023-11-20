#include "shell/plugins/text_input/text_input.h"
#include "shell/log.h"
#include "shell/plugins/text_input/args.h"
#include "shell/plugins/text_input/model.h"

#include <xkbcommon/xkbcommon.h>

namespace shell::plugins {

static const auto k_channel = "flutter/textinput";

static const std::string_view k_set_client_method = "TextInput.setClient";
static const std::string_view k_set_editing_state_method =
    "TextInput.setEditingState";
static const std::string_view k_update_editing_state_method =
    "TextInputClient.updateEditingState";
static const std::string_view k_perform_action_method =
    "TextInputClient.performAction";

TextInput::TextInput(std::shared_ptr<Dispatcher> dispatch) : Plugin(dispatch) {
  dispatcher()->on(k_channel, [this](const FlutterPlatformMessage *message) {
    handle_message(message);
  });
}

void TextInput::handle_key_event(uint32_t keycode, uint32_t symbol,
                                 bool pressed) {
  if (!m_active_model)
    return;

  bool changed = false;
  switch (symbol) {
  case XKB_KEY_Left:
    changed = m_active_model->move_cursor_back();
    break;
  case XKB_KEY_Right:
    changed = m_active_model->move_cursor_forward();
    break;
  case XKB_KEY_BackSpace:
    changed = m_active_model->backspace();
    break;
  case XKB_KEY_Home:
    changed = m_active_model->move_cursor_to_beginning();
    break;
  case XKB_KEY_End:
    changed = m_active_model->move_cursor_to_end();
    break;
  case XKB_KEY_Return:
    enter_pressed();
    break;
  default:
    auto code_point = xkb_keysym_to_utf32(symbol);
    if (code_point) {
      m_active_model->add_codepoint(code_point);
      changed = true;
    }
    break;
  }

  if (!changed)
    return;

  publish_state();
}

void TextInput::handle_message(const FlutterPlatformMessage *message) {
  auto method_call = MethodCall(
      reinterpret_cast<const char *>(message->message), message->message_size);

  if (method_call.method() == k_set_client_method) {
    log::info("set client");

    auto args = method_call.args<SetClientArgs>();
    m_active_model = std::make_unique<TextModel>();
    m_active_client_id = args->client_id;
    m_active_input_action = args->input_action;
    m_active_input_type = args->input_type;

    log::info("client id {}", args->client_id);
    log::info("keyboard appearance {}", args->keyboard_appearance);
  } else if (method_call.method() == k_set_editing_state_method) {
    auto args = method_call.args<SetEditingStateArgs>();

    // Flutter uses -1/-1 for invalid; translate that to 0/0 for the model.
    int base = args->selectionBase;
    int extent = args->selectionExtent;
    if (base == -1 && extent == -1) {
      base = extent = 0;
    }

    m_active_model->set_text(args->text);
    m_active_model->set_selection(TextRange(base, extent));
  }
}

void TextInput::publish_state() {
  if (!m_active_model)
    return;

  auto args = std::make_shared<UpdateEditingStateArgs>();

  args->client_id = m_active_client_id;
  args->composing_base = -1;
  args->composing_extent = -1;

  args->selection_affinity =
      std::make_shared<std::string>("TextAffinity.downstream");
  args->selection_base = m_active_model->selection().base();
  args->selection_extent = m_active_model->selection().extent();

  args->is_directional = false;

  args->text = std::make_shared<std::string>(m_active_model->get_text());

  dispatcher()->call(k_channel, std::make_unique<MethodCall>(
                                    k_update_editing_state_method, args));
}

void TextInput::enter_pressed() {
  if (!m_active_model)
    return;

  if (m_active_input_type == "TextInputType.multiline") {
    m_active_model->add_codepoint('\n');
    publish_state();
  }

  auto args = std::make_shared<PerformActionArgs>();

  args->client_id = m_active_client_id;
  args->input_action = m_active_input_action;

  dispatcher()->call(
      k_channel, std::make_unique<MethodCall>(k_perform_action_method, args));
}

} // namespace shell::plugins