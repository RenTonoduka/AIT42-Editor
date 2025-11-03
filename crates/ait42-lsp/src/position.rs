//! Position Mapping
//!
//! Converts between buffer positions (byte offsets) and LSP positions (line/character).

use lsp_types::Position;

/// Convert buffer byte position to LSP position
///
/// Buffer uses byte offsets (0-indexed), LSP uses (line, character) pairs where:
/// - Lines are 0-indexed
/// - Characters are UTF-16 code units (important for non-ASCII text!)
///
/// # Arguments
/// * `buffer` - The buffer to map from
/// * `pos` - Byte offset in the buffer
///
/// # Returns
/// LSP Position with line and character
pub fn buffer_pos_to_lsp(buffer: &impl BufferLike, pos: usize) -> Position {
    let (line, col) = buffer.pos_to_line_col(pos);

    // Get the line text to calculate UTF-16 offset
    let line_text = buffer.line(line).unwrap_or_default();

    // Convert character column to UTF-16 code units
    let character = utf8_to_utf16_offset(&line_text, col);

    Position {
        line: line as u32,
        character,
    }
}

/// Convert LSP position to buffer byte position
///
/// # Arguments
/// * `buffer` - The buffer to map to
/// * `pos` - LSP position (line, UTF-16 character)
///
/// # Returns
/// Byte offset in the buffer
pub fn lsp_pos_to_buffer(buffer: &impl BufferLike, pos: Position) -> usize {
    let line = pos.line as usize;
    let character = pos.character;

    // Get the line text
    let line_text = buffer.line(line).unwrap_or_default();

    // Convert UTF-16 offset to UTF-8 character column
    let col = utf16_to_utf8_offset(&line_text, character);

    // Convert (line, col) to byte position
    buffer.line_col_to_pos(line, col).unwrap_or(0)
}

/// Convert UTF-8 character offset to UTF-16 code units
fn utf8_to_utf16_offset(text: &str, char_offset: usize) -> u32 {
    let mut utf16_offset = 0;

    for (current_char, ch) in text.chars().enumerate() {
        if current_char >= char_offset {
            break;
        }

        // Each character is 1 or 2 UTF-16 code units
        utf16_offset += ch.len_utf16() as u32;
    }

    utf16_offset
}

/// Convert UTF-16 code unit offset to UTF-8 character offset
fn utf16_to_utf8_offset(text: &str, utf16_offset: u32) -> usize {
    let mut current_utf16 = 0u32;
    let mut char_offset = 0;

    for ch in text.chars() {
        if current_utf16 >= utf16_offset {
            break;
        }

        current_utf16 += ch.len_utf16() as u32;
        char_offset += 1;
    }

    char_offset
}

/// Trait for buffer-like types that can be used for position mapping
pub trait BufferLike {
    /// Convert byte position to (line, column)
    fn pos_to_line_col(&self, pos: usize) -> (usize, usize);

    /// Convert (line, column) to byte position
    fn line_col_to_pos(&self, line: usize, col: usize) -> Option<usize>;

    /// Get line text by index
    fn line(&self, index: usize) -> Option<String>;
}

// Implement for ait42_core::Buffer (when available)
// This will be a blanket implementation

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBuffer {
        content: String,
    }

    impl MockBuffer {
        fn new(content: &str) -> Self {
            Self {
                content: content.to_string(),
            }
        }

        fn lines(&self) -> Vec<&str> {
            self.content.lines().collect()
        }
    }

    impl BufferLike for MockBuffer {
        fn pos_to_line_col(&self, pos: usize) -> (usize, usize) {
            let mut current_pos = 0;
            for (line_idx, line) in self.content.lines().enumerate() {
                let line_end = current_pos + line.len();
                if pos <= line_end {
                    return (line_idx, pos - current_pos);
                }
                current_pos = line_end + 1; // +1 for newline
            }
            (0, 0)
        }

        fn line_col_to_pos(&self, line: usize, col: usize) -> Option<usize> {
            let mut pos = 0;
            for (idx, line_text) in self.content.lines().enumerate() {
                if idx == line {
                    return Some(pos + col.min(line_text.len()));
                }
                pos += line_text.len() + 1; // +1 for newline
            }
            None
        }

        fn line(&self, index: usize) -> Option<String> {
            self.lines().get(index).map(|s| s.to_string())
        }
    }

    #[test]
    fn test_simple_ascii() {
        let buffer = MockBuffer::new("Hello\nWorld");

        // "Hello" at position 0
        let pos = buffer_pos_to_lsp(&buffer, 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);

        // "World" starts at position 6 (after "Hello\n")
        let pos = buffer_pos_to_lsp(&buffer, 6);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 0);

        // Reverse conversion
        let byte_pos = lsp_pos_to_buffer(&buffer, Position::new(1, 0));
        assert_eq!(byte_pos, 6);
    }

    #[test]
    fn test_utf8_characters() {
        let buffer = MockBuffer::new("Hello 世界\nWorld");

        // "世" is at position 6 (after "Hello ")
        // In UTF-8: "世" is 3 bytes, but in UTF-16 it's 1 code unit
        let pos = buffer_pos_to_lsp(&buffer, 6);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 6);
    }

    #[test]
    fn test_emoji() {
        // Emoji "😀" is 4 bytes in UTF-8, 2 UTF-16 code units
        let buffer = MockBuffer::new("Hi 😀\nWorld");

        let pos = buffer_pos_to_lsp(&buffer, 3);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 3);

        // After emoji
        let pos = buffer_pos_to_lsp(&buffer, 7);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 5); // 3 (Hi ) + 2 (emoji in UTF-16)
    }

    #[test]
    fn test_utf16_conversion() {
        assert_eq!(utf8_to_utf16_offset("Hello", 5), 5);
        assert_eq!(utf8_to_utf16_offset("😀", 1), 2); // Emoji is 2 UTF-16 units
        assert_eq!(utf8_to_utf16_offset("Hi 😀", 4), 5); // 3 + 2

        assert_eq!(utf16_to_utf8_offset("Hello", 5), 5);
        assert_eq!(utf16_to_utf8_offset("😀", 2), 1); // 2 UTF-16 units = 1 char
        assert_eq!(utf16_to_utf8_offset("Hi 😀", 5), 4); // 5 UTF-16 units = 4 chars
    }

    #[test]
    fn test_multiline() {
        let buffer = MockBuffer::new("Line 1\nLine 2\nLine 3");

        // Start of line 2
        let pos = Position::new(1, 0);
        let byte_pos = lsp_pos_to_buffer(&buffer, pos);
        assert_eq!(byte_pos, 7); // After "Line 1\n"

        // Round trip
        let pos2 = buffer_pos_to_lsp(&buffer, byte_pos);
        assert_eq!(pos, pos2);
    }
}
