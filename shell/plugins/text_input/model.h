#ifndef ISABEL_SHELL_PLUGIN_TEXT_INPUT_MODEL_H
#define ISABEL_SHELL_PLUGIN_TEXT_INPUT_MODEL_H

#include <algorithm>
#include <cstdint>
#include <string>

namespace shell::plugins {

// A directional range of text.
//
// A |TextRange| describes a range of text with |base| and |extent| positions.
// In the case where |base| == |extent|, the range is said to be collapsed, and
// when |base| > |extent|, the range is said to be reversed.
class TextRange {
public:
  explicit TextRange(size_t position) : m_base(position), m_extent(position) {}
  explicit TextRange(size_t base, size_t extent)
      : m_base(base), m_extent(extent) {}
  TextRange(const TextRange &) = default;
  TextRange &operator=(const TextRange &) = default;

  virtual ~TextRange() = default;

  // The base position of the range.
  size_t base() const { return m_base; }

  // Sets the base position of the range.
  void set_base(size_t pos) { m_base = pos; }

  // The extent position of the range.
  size_t extent() const { return m_extent; }

  // Sets the extent position of the range.
  void set_extent(size_t pos) { m_extent = pos; }

  // The lesser of the base and extent positions.
  size_t start() const { return std::min(m_base, m_extent); }

  // Sets the start position of the range.
  void set_start(size_t pos) {
    if (m_base <= m_extent) {
      m_base = pos;
    } else {
      m_extent = pos;
    }
  }

  // The greater of the base and extent positions.
  size_t end() const { return std::max(m_base, m_extent); }

  // Sets the end position of the range.
  void set_end(size_t pos) {
    if (m_base <= m_extent) {
      m_extent = pos;
    } else {
      m_base = pos;
    }
  }

  // The position of a collapsed range.
  //
  // Asserts that the range is of length 0.
  size_t position() const { return m_extent; }

  // The length of the range.
  size_t length() const { return end() - start(); }

  // Returns true if the range is of length 0.
  bool collapsed() const { return m_base == m_extent; }

  // Returns true if the base is greater than the extent.
  bool reversed() const { return m_base > m_extent; }

  // Returns true if |position| is contained within the range.
  bool contains(size_t position) const {
    return position >= start() && position <= end();
  }

  // Returns true if |range| is contained within the range.
  bool contains(const TextRange &range) const {
    return range.start() >= start() && range.end() <= end();
  }

  bool operator==(const TextRange &other) const {
    return m_base == other.m_base && m_extent == other.m_extent;
  }

private:
  size_t m_base, m_extent;
};

class TextModel {
public:
  TextModel();
  ~TextModel();

  TextRange selection() { return m_selection; }

  void set_text(const std::string &text);
  std::string get_text() const;

  bool set_selection(const TextRange &range);

  void add_text(std::u16string);
  void add_codepoint(uint32_t);

  bool move_cursor_back();
  bool move_cursor_forward();

  TextRange text_range() { return TextRange(0, m_text.length()); }

  bool composing() const { return m_composing; }

private:
  bool delete_selected();

  TextRange editable_range() {
    return m_composing ? m_composing_range : text_range();
  }

  std::u16string m_text;

  TextRange m_selection{0};
  TextRange m_composing_range{0};

  bool m_composing{false};
};
} // namespace shell::plugins

#endif // ISABEL_SHELL_PLUGIN_TEXT_INPUT_MODEL_H
