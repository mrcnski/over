//! Character stream used for parsing.

use std::{
    cell::RefCell,
    fs::File,
    io::{self, Read},
    iter::Peekable,
    mem,
    rc::Rc,
    str::Chars,
};

#[derive(Clone, Debug)]
struct Inner {
    file: Option<String>,
    contents: String,
    stream: Peekable<Chars<'static>>,
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
        let chars: Chars = unsafe { mem::transmute(contents.chars()) };
        let stream = chars.peekable();

        Ok(CharStream {
            inner: Rc::new(RefCell::new(Inner {
                file,
                contents,
                stream,
                line: 1,
                col: 1,
            })),
        })
    }

    pub fn peek(&self) -> Option<char> {
        let mut inner = self.inner.borrow_mut();
        let opt = inner.stream.peek();

        opt.copied()
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

    fn set_line(&mut self, value: usize) {
        let mut inner = self.inner.borrow_mut();
        inner.line = value;
    }

    fn set_col(&mut self, value: usize) {
        let mut inner = self.inner.borrow_mut();
        inner.col = value;
    }
}

impl Iterator for CharStream {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let opt = {
            let mut inner = self.inner.borrow_mut();
            inner.stream.next()
        };

        match opt {
            Some(ch) => {
                if ch == '\n' {
                    let line = self.line();
                    self.set_line(line + 1);
                    self.set_col(1);
                } else {
                    let col = self.col();
                    self.set_col(col + 1);
                }
                Some(ch)
            }
            None => None,
        }
    }
}
