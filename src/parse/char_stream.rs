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
