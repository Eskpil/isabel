use std::string::ToString;

#[derive(Debug, Clone, Copy, Default)]
pub struct TextRange {
    start: usize,
    end: usize,
}

impl TextRange {
    pub fn new(base: usize, extent: usize) -> Self {
        Self {
            start: base,
            end: extent,
        }
    }

    pub fn new_position(position: usize) -> Self {
        TextRange {
            start: position,
            end: position,
        }
    }

    pub fn collapsed(&self) -> bool {
        self.start == self.end
    }

    pub fn length(&self) -> usize {
        self.end - self.start
    }

    pub fn contains(&self, other: &TextRange) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn set_end(&mut self, new_end: usize) {
        self.end = new_end;
    }
}

#[derive(Debug, Clone)]
pub struct TextModel {
    text: String,
    pub selection: TextRange,
    pub composing_range: TextRange,
}

impl TextModel {
    pub fn new() -> Self {
        TextModel {
            text: String::new(),
            selection: TextRange::default(),
            composing_range: TextRange::default(),
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.selection = TextRange::default();
        self.composing_range = TextRange::default();
    }

    pub fn set_selection(&mut self, range: TextRange) -> bool {
        if !self.editable_range().contains(&range) {
            return false;
        }
        self.selection = range;
        true
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    }

    pub fn add_text(&mut self, text: &str) {
        self.delete_selected();
        let position = self.selection.start();
        self.text.insert_str(self.byte_position(position), text);
        self.selection = TextRange::new_position(position + text.chars().count());
    }

    pub fn add_codepoint(&mut self, ch: char) {
        self.add_text(&ch.to_string());
    }

    pub fn move_cursor_forward(&mut self) -> bool {
        if !self.selection.collapsed() {
            self.selection = TextRange::new_position(self.selection.end());
            return true;
        }

        let position = self.selection.start();
        if position < self.text.chars().count() {
            let next_position = position + 1;
            self.selection = TextRange::new_position(next_position);
            return true;
        }

        false
    }

    pub fn move_cursor_back(&mut self) -> bool {
        if !self.selection.collapsed() {
            self.selection = TextRange::new_position(self.selection.start());
            return true;
        }

        let position = self.selection.start();
        if position > 0 {
            let prev_position = position - 1;
            self.selection = TextRange::new_position(prev_position);
            return true;
        }

        false
    }

    pub fn move_cursor_to_beginning(&mut self) -> bool {
        let min_pos = self.editable_range().start();
        if self.selection.collapsed() && self.selection.start() == min_pos {
            return false;
        }
        self.selection = TextRange::new_position(min_pos);
        true
    }

    pub fn move_cursor_to_end(&mut self) -> bool {
        let max_pos = self.editable_range().end();
        if self.selection.collapsed() && self.selection.start() == max_pos {
            return false;
        }
        self.selection = TextRange::new_position(max_pos);
        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.delete_selected() {
            return true;
        }

        let position = self.selection.start();
        if position > 0 {
            let byte_position = self.byte_position(position);
            let prev_char_len = self.text[..byte_position]
                .chars()
                .last()
                .unwrap()
                .len_utf8();
            self.text
                .replace_range(byte_position - prev_char_len..byte_position, "");
            self.selection = TextRange::new_position(position - 1);
            return true;
        }

        false
    }

    fn delete_selected(&mut self) -> bool {
        if self.selection.collapsed() {
            return false;
        }

        let start = self.selection.start();
        let len = self.selection.length();
        let start_byte = self.byte_position(start);
        let end_byte = self.byte_position(start + len);
        self.text.replace_range(start_byte..end_byte, "");
        self.selection = TextRange::new_position(start);

        true
    }

    fn byte_position(&self, char_position: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_position)
            .map(|(byte_pos, _)| byte_pos)
            .unwrap_or_else(|| self.text.len())
    }

    // Dummy functions for placeholders
    fn editable_range(&self) -> TextRange {
        // Add logic to return editable range
        TextRange::new_position(0)
    }
}
