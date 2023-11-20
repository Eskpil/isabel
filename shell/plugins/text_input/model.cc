#include <codecvt>
#include <locale>

#include "shell/plugins/text_input/model.h"

namespace shell::plugins {

// Returns true if |code_point| is a leading surrogate of a surrogate pair.
static bool is_leading_surrogate(char32_t code_point) {
  return (code_point & 0xFFFFFC00) == 0xD800;
}

// Returns true if |code_point| is a trailing surrogate of a surrogate pair.
static bool is_trailing_surrogate(char32_t code_point) {
  return (code_point & 0xFFFFFC00) == 0xDC00;
}

TextModel::TextModel() {}
TextModel::~TextModel() {}

void TextModel::set_text(const std::string &text) {
  std::wstring_convert<std::codecvt_utf8_utf16<char16_t>, char16_t>
      utf16_converter;
  m_text = utf16_converter.from_bytes(text);
  m_selection = TextRange(0);
  m_composing_range = TextRange(0);
}

bool TextModel::set_selection(const shell::plugins::TextRange &range) {
  if (composing() && !range.collapsed()) {
    return false;
  }
  if (!editable_range().contains(range)) {
    return false;
  }
  m_selection = range;
  return true;
}

std::string TextModel::get_text() const {
  std::wstring_convert<std::codecvt_utf8_utf16<char16_t>, char16_t>
      utf8_converter;
  return utf8_converter.to_bytes(m_text);
}

void TextModel::add_text(std::u16string text) {
  delete_selected();
  if (composing()) {
    // Delete the current composing text, set the cursor to composing start.
    m_text.erase(m_composing_range.start(), m_composing_range.length());
    m_selection = TextRange(m_composing_range.start());
    m_composing_range.set_end(m_composing_range.start() + text.length());
  }
  size_t position = m_selection.position();
  m_text.insert(position, text);
  m_selection = TextRange(position + text.length());
}

void TextModel::add_codepoint(uint32_t c) {
  if (c <= 0xFFFF) {
    add_text(std::u16string({static_cast<char16_t>(c)}));
  } else {
    char32_t to_decompose = c - 0x10000;
    add_text(std::u16string({
        // High surrogate.
        static_cast<char16_t>((to_decompose >> 10) + 0xd800),
        // Low surrogate.
        static_cast<char16_t>((to_decompose % 0x400) + 0xdc00),
    }));
  }
}

bool TextModel::move_cursor_forward() {
  // If there's a selection, move to the end of the selection.
  if (!m_selection.collapsed()) {
    m_selection = TextRange(m_selection.end());
    return true;
  }
  // Otherwise, move the cursor forward.
  size_t position = m_selection.position();
  if (position != editable_range().end()) {
    int count = is_leading_surrogate(m_text.at(position)) ? 2 : 1;
    m_selection = TextRange(position + count);
    return true;
  }

  return false;
}

bool TextModel::move_cursor_back() {
  if (!m_selection.collapsed()) {
    m_selection = TextRange(m_selection.start());
    return true;
  }
  // Otherwise, move the cursor backward.
  size_t position = m_selection.position();
  if (position != editable_range().start()) {
    int count = is_trailing_surrogate(m_text.at(position - 1)) ? 2 : 1;
    m_selection = TextRange(position - count);
    return true;
  }
  return false;
}

bool TextModel::move_cursor_to_beginning() {
  size_t min_pos = editable_range().start();
  if (m_selection.collapsed() && m_selection.position() == min_pos) {
    return false;
  }
  m_selection = TextRange(min_pos);
  return true;
}

bool TextModel::move_cursor_to_end() {
  size_t max_pos = editable_range().end();
  if (m_selection.collapsed() && m_selection.position() == max_pos) {
    return false;
  }
  m_selection = TextRange(max_pos);
  return true;
}

bool TextModel::backspace() {
  if (delete_selected()) {
    return true;
  }
  // There is no selection. Delete the preceding codepoint.
  size_t position = m_selection.position();
  if (position != editable_range().start()) {
    int count = is_trailing_surrogate(m_text.at(position - 1)) ? 2 : 1;
    m_text.erase(position - count, count);
    m_selection = TextRange(position - count);
    if (composing()) {
      m_composing_range.set_end(m_composing_range.end() - count);
    }
    return true;
  }

  return false;
}

bool TextModel::delete_selected() {
  if (m_selection.collapsed()) {
    return false;
  }

  size_t start = m_selection.start();
  m_text.erase(start, m_selection.length());
  m_selection = TextRange(start);
  if (composing()) {
    // This occurs only immediately after composing has begun with a selection.
    m_composing_range = m_selection;
  }

  return true;
}

} // namespace shell::plugins