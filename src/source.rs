use codespan_reporting::files::{line_starts, Files};
use std::{cmp::Ordering, fmt, sync::Arc};

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
    /// The starting byte indices in the source code.
    line_starts: Vec<usize>,
}

impl File {
    pub fn new(name: Arc<str>, source: Arc<str>) -> Self {
        Self {
            name,
            line_starts: line_starts(&source).collect(),
            source,
        }
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
}

/// The implementation for the `source` query.
fn source(db: &dyn SourceDatabase, file: FileId) -> Arc<str> {
    let file = db.lookup_intern_file(file);
    Arc::clone(&file.source)
}

impl<'a> Files<'a> for dyn SourceDatabase {
    type FileId = FileId;
    type Name = Arc<str>;
    type Source = Arc<str>;

    fn name(&'a self, id: Self::FileId) -> Option<Self::Name> {
        let file = self.lookup_intern_file(id);
        let name = Arc::clone(&file.name);
        Some(name)
    }

    fn source(&'a self, id: Self::FileId) -> Option<Self::Source> {
        Some(self.source(id))
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Option<usize> {
        let file = self.lookup_intern_file(id);
        match file.line_starts.binary_search(&byte_index) {
            Ok(line) => Some(line),
            Err(next_line) => Some(next_line - 1),
        }
    }

    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Option<std::ops::Range<usize>> {
        let file = self.lookup_intern_file(id);

        let line_start = |file: &File, idx: usize| match idx.cmp(&file.line_starts.len()) {
            Ordering::Less => file.line_starts.get(idx).cloned(),
            Ordering::Equal => Some(file.source.len()),
            Ordering::Greater => None,
        };

        let line = line_start(&file, line_index)?;
        let next_line = line_start(&file, line_index + 1)?;

        Some(line..next_line)
    }
}
