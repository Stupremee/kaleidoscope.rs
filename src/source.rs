use smol_str::SmolStr;
use std::{cmp::Ordering, fmt, ops::Range, sync::Arc};

/// An interned file, which can be resolved using the `SourceDatabase`.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct FileId(salsa::InternId);

impl Default for FileId {
    fn default() -> Self {
        Self(0usize.into())
    }
}

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
    name: Arc<SmolStr>,
    /// The source of this file.
    source: Arc<String>,
}

impl File {
    pub fn new(name: Arc<SmolStr>, source: Arc<String>) -> Self {
        Self { name, source }
    }
}

#[salsa::query_group(SourceDatabaseStorage)]
pub trait SourceDatabase: salsa::Database {
    /// Interns the given `File` and returns a `FileId` which can later be used to
    /// lookup the `File` using `lookup_intern_file`.
    #[salsa::interned]
    fn intern_file(&self, file: File) -> FileId;

    /// Looks up the given `FileId` and then returns a reference to the source of
    /// the File.
    fn source(&self, file: FileId) -> Arc<String>;

    /// Looks up the given `FileId` and then returns a reference to the name of
    /// the File.
    fn name(&self, file: FileId) -> Arc<SmolStr>;

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
fn source(db: &dyn SourceDatabase, file: FileId) -> Arc<String> {
    let file = db.lookup_intern_file(file);
    Arc::clone(&file.source)
}

fn name(db: &dyn SourceDatabase, file: FileId) -> Arc<SmolStr> {
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

/// A atomic counted reference to a `String`, which implements `AsRef<str>`
#[derive(Debug)]
pub struct StringRef {
    string: Arc<String>,
}

impl AsRef<str> for StringRef {
    fn as_ref(&self) -> &str {
        self.string.as_ref()
    }
}

/// A cache that can be used as an [`Files`] implementation
/// and will use a `SourceDatabase` to get the information.
///
/// This allows caching expensive things like line ranges or indicies.
///
/// Thanks to [`Kixiron`] for this suggestion.
///
/// [`Kixiron`]: https://github.com/Kixiron
#[derive(Clone)]
pub struct FileCache<'db> {
    db: &'db dyn SourceDatabase,
}

impl<'db> FileCache<'db> {
    pub fn new(db: &'db dyn SourceDatabase) -> Self {
        Self { db }
    }
}

impl<'a> codespan_reporting::files::Files<'a> for FileCache<'a> {
    type FileId = FileId;
    type Name = Arc<SmolStr>;
    type Source = StringRef;

    fn name(&'a self, id: Self::FileId) -> Option<Self::Name> {
        Some(self.db.name(id))
    }

    fn source(&'a self, id: Self::FileId) -> Option<Self::Source> {
        let source = self.db.source(id);
        Some(StringRef { string: source })
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Option<usize> {
        self.db.line_index(id, byte_index)
    }

    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Option<Range<usize>> {
        self.db.line_range(id, line_index)
    }
}
