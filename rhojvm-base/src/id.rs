use std::hash::{Hash, Hasher};

#[derive(Copy, Clone)]
pub struct ClassId(u32);
impl ClassId {
    #[must_use]
    pub fn new_unchecked(id: u32) -> ClassId {
        ClassId(id)
    }

    #[must_use]
    pub fn get(self) -> u32 {
        self.0
    }
}

// This only really holds true if they're from the same [`ClassNames`] instance
impl PartialEq for ClassId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for ClassId {}
impl Hash for ClassId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.0)
    }
}
#[cfg(feature = "implementation-cheaper-map-hashing")]
impl nohash_hasher::IsEnabled for ClassId {}

impl std::fmt::Debug for ClassId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ClassId({})", self.0))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PackageId(u32);
impl PackageId {
    pub(crate) fn new_unchecked(id: u32) -> PackageId {
        PackageId(id)
    }
}

/// This is an index into the methods
/// This is not meaningful without a class
pub type MethodIndex = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExactMethodId {
    class_id: ClassId,
    method_index: MethodIndex,
}
impl ExactMethodId {
    #[must_use]
    pub fn unchecked_compose(class_id: ClassId, method_index: MethodIndex) -> Self {
        Self {
            class_id,
            method_index,
        }
    }

    #[must_use]
    pub fn decompose(self) -> (ClassId, MethodIndex) {
        (self.class_id, self.method_index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MethodId {
    Exact(ExactMethodId),
    ArrayClone,
}
impl MethodId {
    #[must_use]
    pub fn unchecked_compose(class_id: ClassId, method_index: MethodIndex) -> Self {
        MethodId::Exact(ExactMethodId::unchecked_compose(class_id, method_index))
    }

    #[must_use]
    pub fn decompose(self) -> Option<(ClassId, MethodIndex)> {
        self.into_exact().map(ExactMethodId::decompose)
    }

    #[must_use]
    pub fn into_exact(self) -> Option<ExactMethodId> {
        match self {
            MethodId::Exact(x) => Some(x),
            MethodId::ArrayClone => None,
        }
    }
}
impl From<ExactMethodId> for MethodId {
    fn from(v: ExactMethodId) -> Self {
        MethodId::Exact(v)
    }
}

pub(crate) fn is_array_class(first: &str) -> bool {
    first.starts_with('[')
}

pub(crate) fn is_array_class_bytes(first: &[u8]) -> bool {
    first.starts_with(&[b'['])
}
