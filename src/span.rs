//! Primitives to locate data in a file.

use crate::source::FileId;
use std::ops::{Deref, DerefMut, Index, Range};

/// A span in a file that has a start and end index.
///
/// Most of the API is a reimplementation of the [`codespan::Span`].
///
/// [`codespan::Span`]: https://docs.rs/codespan/0.9.5/codespan/struct.Span.html
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    /// Creates a new `Span`.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// The `start` of self.
    pub fn start(&self) -> usize {
        self.start
    }

    /// The end of `self`.
    pub fn end(&self) -> usize {
        self.end
    }

    /// Merge two spans together.
    pub fn merge(self, other: Self) -> Self {
        let start = self.start.min(other.start);
        let end = self.end.max(other.end);
        Self::new(start, end)
    }

    /// A helper function to tell whether two spans do not overlap.
    pub fn disjoint(&self, other: &Span) -> bool {
        let (first, last) = if self.end < other.end {
            (self, other)
        } else {
            (other, self)
        };
        first.end <= last.start
    }

    /// Returns a reference to the data that is located at the span.
    pub fn index_in<'input, I, N>(&self, val: &'input I) -> &'input I::Output
    where
        I: Index<Range<N>>,
        N: From<usize>,
    {
        let start = self.start().into();
        let end = self.end().into();
        val.index(start..end)
    }

    pub fn span<T>(self, file: FileId, data: T) -> Locatable<T> {
        Locatable {
            data,
            span: self,
            file,
        }
    }
}

impl Into<Range<usize>> for Span {
    fn into(self) -> Range<usize> {
        self.start..self.end
    }
}

impl From<Range<usize>> for Span {
    fn from(x: Range<usize>) -> Self {
        Self::new(x.start, x.end)
    }
}

/// Any object that is located in a file at a span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locatable<T> {
    data: T,
    span: Span,
    file: FileId,
}

impl<T> Locatable<T> {
    pub fn new(data: T, span: Span, file: FileId) -> Self {
        Self { data, span, file }
    }

    /// Returns a reference to the data inside `Self`.
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns a exclusive reference to the data inside `Self`.
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Returns a copy of the span of `Self`.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Returns a copy of the file id of `Self`.
    pub fn file(&self) -> FileId {
        self.file
    }

    /// Destructs this `Locatable` into all inner parts.
    pub fn destruct(self) -> (T, Span, FileId) {
        (self.data, self.span, self.file)
    }
}

impl<T> Deref for Locatable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

impl<T> DerefMut for Locatable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap() {
        let first = Span::new(0, 3);
        let second = Span::new(1, 3);
        assert!(!first.disjoint(&second));
    }
}
