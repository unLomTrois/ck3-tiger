//! Contains the core [`Token`] and [`Loc`] types, which represent pieces of game script and where
//! in the game files they came from.

use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt::{Debug, Display, Error, Formatter};
use std::ops::{Bound, Range, RangeBounds};
use std::path::{Path, PathBuf};
use std::slice::SliceIndex;
use std::sync::Arc;

use crate::date::Date;
use crate::fileset::{FileEntry, FileKind};
use crate::pathtable::{PathTable, PathTableIndex};
use crate::report::{error, error_info, untidy, ErrorKey};
use crate::stringtable::StringTable;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Loc {
    pub(crate) idx: PathTableIndex,
    pub kind: FileKind,
    /// line 0 means the loc applies to the file as a whole.
    pub line: u32,
    pub column: u32,
    /// Used in macro expansions to point to the macro invocation
    pub link: Option<Arc<Loc>>,
}

impl Loc {
    #[must_use]
    pub(crate) fn for_file(pathname: PathBuf, kind: FileKind, fullpath: PathBuf) -> Self {
        let idx = PathTable::store(pathname, fullpath);
        Loc { idx, kind, line: 0, column: 0, link: None }
    }

    pub fn filename(&self) -> Cow<str> {
        PathTable::lookup_path(self.idx)
            .file_name()
            .unwrap_or_else(|| OsStr::new(""))
            .to_string_lossy()
    }

    pub fn pathname(&self) -> &'static Path {
        PathTable::lookup_path(self.idx)
    }

    pub fn fullpath(&self) -> &'static Path {
        PathTable::lookup_fullpath(self.idx)
    }

    pub fn same_file(&self, other: &Loc) -> bool {
        self.idx == other.idx
    }
}

impl From<&FileEntry> for Loc {
    fn from(entry: &FileEntry) -> Self {
        if let Some(idx) = entry.path_idx() {
            Loc { idx, kind: entry.kind(), line: 0, column: 0, link: None }
        } else {
            Self::for_file(entry.path().to_path_buf(), entry.kind(), entry.fullpath().to_path_buf())
        }
    }
}

impl From<&mut FileEntry> for Loc {
    fn from(entry: &mut FileEntry) -> Self {
        (&*entry).into()
    }
}

impl From<FileEntry> for Loc {
    fn from(entry: FileEntry) -> Self {
        (&entry).into()
    }
}

impl Debug for Loc {
    /// Roll our own `Debug` implementation to handle the path field
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Loc")
            .field("pathindex", &self.idx)
            .field("pathname", &self.pathname())
            .field("fullpath", &self.fullpath())
            .field("kind", &self.kind)
            .field("line", &self.line)
            .field("column", &self.column)
            .field("link", &self.link)
            .finish()
    }
}

/// A Token consists of a string (stored in a `StringTable`) and its location in the parsed files.
#[derive(Clone, Debug)]
pub struct Token {
    s: &'static str,
    pub loc: Loc,
}

impl Token {
    #[must_use]
    pub fn new(s: &str, loc: Loc) -> Self {
        Token { s: StringTable::store(s), loc }
    }

    #[must_use]
    pub fn from_static_str(s: &'static str, loc: Loc) -> Self {
        Token { s, loc }
    }

    /// Create a `Token` from a substring of the given `Token`.
    #[must_use]
    pub fn subtoken<R>(&self, range: R, loc: Loc) -> Token
    where
        R: RangeBounds<usize> + SliceIndex<str, Output = str>,
    {
        Token { s: &self.s[range], loc }
    }

    /// Create a `Token` from a subtring of the given `Token`,
    /// stripping any whitespace from the created token.
    #[must_use]
    pub fn subtoken_stripped(&self, mut range: Range<usize>, mut loc: Loc) -> Token {
        let mut start = match range.start_bound() {
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i + 1,
            Bound::Unbounded => 0,
        };
        let mut end = match range.end_bound() {
            Bound::Included(&i) => i + 1,
            Bound::Excluded(&i) => i,
            Bound::Unbounded => self.s.len(),
        };
        for (i, c) in self.s[range.clone()].char_indices() {
            if !c.is_whitespace() {
                start += i;
                range = start..end;
                break;
            }
            loc.column += 1;
        }
        for (i, c) in self.s[range.clone()].char_indices().rev() {
            if !c.is_whitespace() {
                end = start + i + c.len_utf8();
                range = start..end;
                break;
            }
        }
        Token { s: &self.s[range], loc }
    }

    pub fn as_str(&self) -> &str {
        self.s
    }

    pub fn is(&self, s: &str) -> bool {
        self.s == s
    }

    pub fn lowercase_is(&self, s: &str) -> bool {
        self.s.to_lowercase() == s
    }

    pub fn starts_with(&self, s: &str) -> bool {
        self.s.starts_with(s)
    }

    #[must_use]
    pub fn split(&self, ch: char) -> Vec<Token> {
        let mut pos = 0;
        let mut vec = Vec::new();
        let mut loc = self.loc.clone();
        let mut lines: u32 = 0;
        for (cols, (i, c)) in self.s.char_indices().enumerate() {
            let cols = u32::try_from(cols).expect("internal error: 4GB token");
            if c == ch {
                vec.push(self.subtoken(pos..i, loc.clone()));
                pos = i + 1;
                loc.column = self.loc.column + cols + 1;
                loc.line = self.loc.line + lines;
            }
            if c == '\n' {
                lines += 1;
            }
        }
        vec.push(self.subtoken(pos.., loc));
        vec
    }

    #[must_use]
    pub fn strip_suffix(&self, sfx: &str) -> Option<Token> {
        self.s.strip_suffix(sfx).map(|pfx| Token::from_static_str(pfx, self.loc.clone()))
    }

    #[must_use]
    pub fn strip_prefix(&self, pfx: &str) -> Option<Token> {
        #[allow(clippy::cast_possible_truncation)]
        self.s.strip_prefix(pfx).map(|sfx| {
            let mut loc = self.loc.clone();
            loc.column += pfx.chars().count() as u32;
            Token::from_static_str(sfx, loc)
        })
    }

    #[must_use]
    pub fn split_once(&self, ch: char) -> Option<(Token, Token)> {
        for (cols, (i, c)) in self.s.char_indices().enumerate() {
            let cols = u32::try_from(cols).expect("internal error: 4GB token");
            if c == ch {
                let token1 = self.subtoken(..i, self.loc.clone());
                let mut loc = self.loc.clone();
                loc.column += cols + 1;
                let token2 = self.subtoken(i + 1.., loc);
                return Some((token1, token2));
            }
        }
        None
    }

    /// Split the token at the first instance of ch, such that ch is part of the first returned token.
    #[must_use]
    pub fn split_after(&self, ch: char) -> Option<(Token, Token)> {
        for (cols, (i, c)) in self.s.char_indices().enumerate() {
            let cols = u32::try_from(cols).expect("internal error: 4GB token");
            #[allow(clippy::cast_possible_truncation)] // chlen can't be more than 6
            if c == ch {
                let chlen = ch.len_utf8();
                let token1 = self.subtoken(..i + chlen, self.loc.clone());
                let mut loc = self.loc.clone();
                loc.column += cols + chlen as u32;
                let token2 = self.subtoken(i + chlen.., loc);
                return Some((token1, token2));
            }
        }
        None
    }

    pub fn combine(&mut self, other: &Token, c: char) {
        let mut s = self.s.to_string();
        s.push(c);
        s.push_str(other.s);
        self.s = StringTable::store(&s);
    }

    #[must_use]
    pub fn trim(&self) -> Token {
        let mut real_start = None;
        let mut real_end = self.s.len();
        for (cols, (i, c)) in self.s.char_indices().enumerate() {
            let cols = u32::try_from(cols).expect("internal error: 4GB token");
            if c != ' ' {
                real_start = Some((cols, i));
                break;
            }
        }
        // looping over the indices is safe here because we're only skipping spaces
        while real_end > 0 && &self.s[real_end - 1..real_end] == " " {
            real_end -= 1;
        }
        if let Some((cols, i)) = real_start {
            let mut loc = self.loc.clone();
            loc.column += cols;
            self.subtoken(i..real_end, loc)
        } else {
            // all spaces
            Token::from_static_str("", self.loc.clone())
        }
    }

    pub fn expect_number(&self) -> Option<f64> {
        self.check_number();
        if let Ok(v) = self.s.parse::<f64>() {
            Some(v)
        } else {
            error(self, ErrorKey::Validation, "expected number");
            None
        }
    }

    pub fn get_number(&self) -> Option<f64> {
        self.s.parse::<f64>().ok()
    }

    pub fn is_number(&self) -> bool {
        self.s.parse::<f64>().is_ok()
    }

    pub fn check_number(&self) {
        if let Some(idx) = self.s.find('.') {
            if self.s.len() - idx > 6 {
                let msg = "only 5 decimals are supported";
                let info =
                    "if you give more decimals, you get an error and the number is read as 0";
                error_info(self, ErrorKey::Validation, msg, info);
            }
        }
    }

    /// Some files seem not to have the 5-decimal limitation
    pub fn expect_precise_number(&self) -> Option<f64> {
        if let Ok(v) = self.s.parse::<f64>() {
            Some(v)
        } else {
            error(self, ErrorKey::Validation, "expected number");
            None
        }
    }

    pub fn expect_integer(&self) -> Option<i64> {
        if let Ok(v) = self.s.parse::<i64>() {
            Some(v)
        } else {
            error(self, ErrorKey::Validation, "expected integer");
            None
        }
    }

    pub fn get_integer(&self) -> Option<i64> {
        self.s.parse::<i64>().ok()
    }

    pub fn is_integer(&self) -> bool {
        self.s.parse::<i64>().is_ok()
    }

    pub fn expect_date(&self) -> Option<Date> {
        if let Ok(v) = self.s.parse::<Date>() {
            if self.s.ends_with('.') {
                untidy(ErrorKey::Validation).msg("trailing dot on date").loc(self).push();
            }
            Some(v)
        } else {
            error(self, ErrorKey::Validation, "expected date");
            None
        }
    }

    pub fn get_date(&self) -> Option<Date> {
        self.s.parse::<Date>().ok()
    }

    pub fn is_date(&self) -> bool {
        self.s.parse::<Date>().is_ok()
    }

    #[must_use]
    pub fn linked(mut self, link: Option<Arc<Loc>>) -> Self {
        self.loc.link = link;
        self
    }
}

/// Tokens are compared for equality regardless of their loc.
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.s == other.s
    }
}

impl Eq for Token {}

impl From<Loc> for Token {
    fn from(loc: Loc) -> Self {
        Token { s: "", loc }
    }
}

impl From<&Loc> for Token {
    fn from(loc: &Loc) -> Self {
        Token { s: "", loc: loc.clone() }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.s)
    }
}
