//! Character stream used for parsing.

use std::{
    cell::RefCell,
    fs::File,
    io::{self, Read},
    rc::Rc,
};

#[derive(Clone, Debug)]
struct Inner {
    file: Option<String>,
    contents: String,
    // Current byte position in `contents`.
    pos: usize,
    line: usize,
    col: usize,
}

#[derive(Clone, Debug)]
pub struct CharStream {
    inner: Rc<RefCell<Inner>>,
}

impl CharStream {
    pub fn from_file(path: &str) -> io::Result<CharStream> {
        let mut file = File::open(path)?;

        let len = file.metadata()?.len();
        let mut contents = String::with_capacity(len as usize);

        file.read_to_string(&mut contents)?;

        Self::from_string_impl(Some(String::from(path)), contents)
    }

    pub fn from_string(contents: String) -> io::Result<CharStream> {
        Self::from_string_impl(None, contents)
    }

    fn from_string_impl(file: Option<String>, contents: String) -> io::Result<CharStream> {
        Ok(CharStream {
            inner: Rc::new(RefCell::new(Inner {
                file,
                contents,
                pos: 0,
                line: 1,
                col: 1,
            })),
        })
    }

    pub fn peek(&self) -> Option<char> {
        let inner = self.inner.borrow();
        inner.contents[inner.pos..].chars().next()
    }

    pub fn file(&self) -> Option<String> {
        let inner = self.inner.borrow();
        inner.file.clone()
    }

    pub fn line(&self) -> usize {
        let inner = self.inner.borrow();
        inner.line
    }

    pub fn col(&self) -> usize {
        let inner = self.inner.borrow();
        inner.col
    }

    /// Skips whitespace and comments, stopping at the next significant character.
    /// Returns true if such a character was found or false if we got to the end of the stream.
    pub fn skip_whitespace(&mut self) -> bool {
        let mut guard = self.inner.borrow_mut();
        let Inner {
            ref contents,
            ref mut pos,
            ref mut line,
            ref mut col,
            ..
        } = *guard;
        let bytes = contents.as_bytes();

        while *pos < bytes.len() {
            match bytes[*pos] {
                b'\n' => {
                    *pos += 1;
                    *line += 1;
                    *col = 1;
                }
                b' ' | b'\t' | b'\x0b' | b'\x0c' | b'\r' => {
                    *pos += 1;
                    *col += 1;
                }
                b'#' => {
                    // Comment found; skip the rest of the line.
                    match contents[*pos..].find('\n') {
                        Some(idx) => {
                            *pos += idx + 1;
                            *line += 1;
                            *col = 1;
                        }
                        None => {
                            *col += contents[*pos..].chars().count();
                            *pos = bytes.len();
                            return false;
                        }
                    }
                }
                b if b < 0x80 => return true,
                _ => {
                    let ch = contents[*pos..].chars().next().unwrap();
                    if ch.is_whitespace() {
                        *pos += ch.len_utf8();
                        *col += 1;
                    } else {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Consumes a run of ASCII digits, appending them to `out`.
    /// Returns the number of digits consumed.
    pub fn take_digits(&mut self, out: &mut String) -> usize {
        let mut guard = self.inner.borrow_mut();
        let Inner {
            ref contents,
            ref mut pos,
            ref mut col,
            ..
        } = *guard;
        let bytes = contents.as_bytes();

        let start = *pos;
        while *pos < bytes.len() && bytes[*pos].is_ascii_digit() {
            *pos += 1;
        }

        let count = *pos - start;
        *col += count;
        out.push_str(&contents[start..*pos]);
        count
    }

    /// Consumes a run of characters valid as non-first field characters (see
    /// `Obj::is_valid_field_char`: alphabetic, digit, or '_'), appending them to `out`.
    pub fn take_field_chars(&mut self, out: &mut String) {
        let mut guard = self.inner.borrow_mut();
        let Inner {
            ref contents,
            ref mut pos,
            ref mut col,
            ..
        } = *guard;
        let bytes = contents.as_bytes();

        let start = *pos;
        while *pos < bytes.len() {
            match bytes[*pos] {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' => {
                    *pos += 1;
                    *col += 1;
                }
                b if b < 0x80 => break,
                _ => {
                    let ch = contents[*pos..].chars().next().unwrap();
                    if ch.is_alphabetic() {
                        *pos += ch.len_utf8();
                        *col += 1;
                    } else {
                        break;
                    }
                }
            }
        }
        out.push_str(&contents[start..*pos]);
    }

    /// Consumes characters up to (but not including) the next '"', '\\', or the end of the
    /// stream, appending them to `out`.
    pub fn take_str_span(&mut self, out: &mut String) {
        let mut guard = self.inner.borrow_mut();
        let Inner {
            ref contents,
            ref mut pos,
            ref mut line,
            ref mut col,
            ..
        } = *guard;
        let bytes = contents.as_bytes();

        let start = *pos;
        while *pos < bytes.len() {
            match bytes[*pos] {
                b'"' | b'\\' => break,
                b'\n' => {
                    *pos += 1;
                    *line += 1;
                    *col = 1;
                }
                b if b < 0x80 => {
                    *pos += 1;
                    *col += 1;
                }
                _ => {
                    let ch = contents[*pos..].chars().next().unwrap();
                    *pos += ch.len_utf8();
                    *col += 1;
                }
            }
        }
        out.push_str(&contents[start..*pos]);
    }
}

impl Iterator for CharStream {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let mut inner = self.inner.borrow_mut();

        let ch = inner.contents[inner.pos..].chars().next()?;
        inner.pos += ch.len_utf8();

        if ch == '\n' {
            inner.line += 1;
            inner.col = 1;
        } else {
            inner.col += 1;
        }

        Some(ch)
    }
}
