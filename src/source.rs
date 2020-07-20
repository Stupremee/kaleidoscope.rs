use codespan_reporting::files::Files;
use std::{cmp::Ordering, fmt, ops::Range, sync::Arc};

/// An interned file, which can be resolved using the `SourceDatabase`.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct FileId(salsa::InternId);

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl salsa::InternKey for FileId {
    fn from_intern_id(v: salsa::InternId) -> Self {
        Self(v)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

/// A file that can be interned using the `SourceDatabase`.
///
/// This is mostly a reimplementation of `codespan_reporting::files::SimpleFile`.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct File {
    // I'm not sure if it's the right way, but I want to avoid that salsa
    // clones the whole source code every time you lookup the file.
    /// The `name` of the file.
    name: Arc<str>,
    /// The source of this file.
    source: Arc<str>,
}

impl File {
    pub fn new(name: Arc<str>, source: Arc<str>) -> Self {
        Self { name, source }
    }
}

#[salsa::query_group(SourceDatabaseStorage)]
trait SourceDatabase: salsa::Database {
    /// Interns the given `File` and returns a `FileId` which can later be used to
    /// lookup the `File` using `lookup_intern_file`.
    #[salsa::interned]
    fn intern_file(&self, file: File) -> FileId;

    /// Looks up the given `FileId` and then returns a reference to the source of
    /// the File.
    fn source(&self, file: FileId) -> Arc<str>;

    /// Looks up the given `FileId` and then returns a reference to the name of
    /// the File.
    fn name(&self, file: FileId) -> Arc<str>;

    /// Returns the index of the line at the given byte index in the given file.
    fn line_index(&self, file: FileId, byte_index: usize) -> Option<usize>;

    /// Returns the byte range of the given line in the given file.
    fn line_range(&self, file: FileId, line_index: usize) -> Option<Range<usize>>;

    /// Returns the indices of every line start in the file.
    fn line_starts(&self, file: FileId) -> Arc<Vec<usize>>;

    /// Returns the start index of the line in the file.
    fn line_start(&self, file: FileId, line_index: usize) -> Option<usize>;
}

/// The implementation for the `source` query.
fn source(db: &dyn SourceDatabase, file: FileId) -> Arc<str> {
    let file = db.lookup_intern_file(file);
    Arc::clone(&file.source)
}

fn name(db: &dyn SourceDatabase, file: FileId) -> Arc<str> {
    let file = db.lookup_intern_file(file);
    Arc::clone(&file.name)
}

fn line_starts(db: &dyn SourceDatabase, file: FileId) -> Arc<Vec<usize>> {
    let starts = codespan_reporting::files::line_starts(&db.source(file)).collect();
    Arc::new(starts)
}

fn line_start(db: &dyn SourceDatabase, file: FileId, line_index: usize) -> Option<usize> {
    let len = db.line_starts(file).len();
    match line_index.cmp(&len) {
        Ordering::Less => db.line_starts(file).get(line_index).copied(),
        Ordering::Equal => Some(db.source(file).len()),
        Ordering::Greater => None,
    }
}

fn line_index(db: &dyn SourceDatabase, file: FileId, byte_index: usize) -> Option<usize> {
    match db.line_starts(file).binary_search(&byte_index) {
        Ok(line) => Some(line),
        Err(line) => Some(line - 1),
    }
}

fn line_range(db: &dyn SourceDatabase, file: FileId, line_index: usize) -> Option<Range<usize>> {
    let line = db.line_start(file, line_index)?;
    let next_line = db.line_start(file, line_index + 1)?;
    Some(line..next_line)
}

impl<'a> Files<'a> for dyn SourceDatabase {
    type FileId = FileId;
    type Name = Arc<str>;
    type Source = Arc<str>;

    fn name(&'a self, id: Self::FileId) -> Option<Self::Name> {
        Some(self.name(id))
    }

    fn source(&'a self, id: Self::FileId) -> Option<Self::Source> {
        Some(self.source(id))
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Option<usize> {
        self.line_index(id, byte_index)
    }

    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Option<std::ops::Range<usize>> {
        self.line_range(id, line_index)
    }
}
