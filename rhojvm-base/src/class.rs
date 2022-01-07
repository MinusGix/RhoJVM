use std::path::PathBuf;

use classfile_parser::{
    constant_info::{ClassConstant, ConstantInfo, Utf8Constant},
    constant_pool::{ConstantPoolIndex, ConstantPoolIndexRaw},
    method_info::MethodInfo,
    ClassFile, ClassFileVersion,
};

pub use classfile_parser::ClassAccessFlags;

use crate::{
    code::types::PrimitiveType,
    id::{ClassFileId, ClassId, MethodId, MethodIndex, PackageId},
    BadIdError, ClassNames,
};

#[derive(Debug, Clone)]
pub enum ClassFileIndexError {
    InvalidThisClassIndex,
    InvalidThisClassNameIndex,
    InvalidSuperClassIndex,
    InvalidSuperClassNameIndex,
}

#[derive(Debug, Clone)]
pub struct ClassFileData {
    pub(crate) id: ClassFileId,
    #[allow(dead_code)]
    /// The direct path to the file
    pub(crate) path: PathBuf,
    pub(crate) class_file: ClassFile,
}
impl ClassFileData {
    #[must_use]
    pub fn id(&self) -> ClassFileId {
        self.id
    }

    #[must_use]
    pub fn version(&self) -> Option<ClassFileVersion> {
        Some(self.class_file.version)
    }

    pub fn get_t<'a, T>(&'a self, i: impl TryInto<ConstantPoolIndex<T>>) -> Option<&'a T>
    where
        &'a T: TryFrom<&'a ConstantInfo>,
    {
        self.class_file.const_pool.get_t(i)
    }

    pub fn get_t_mut<'a, T>(
        &'a mut self,
        i: impl TryInto<ConstantPoolIndex<T>>,
    ) -> Option<&'a mut T>
    where
        &'a mut T: TryFrom<&'a mut ConstantInfo>,
    {
        self.class_file.const_pool.get_t_mut(i)
    }

    pub fn get_text_t(&self, i: impl TryInto<ConstantPoolIndex<Utf8Constant>>) -> Option<&str> {
        self.get_t(i).map(|x| x.utf8_string.as_str())
    }

    #[must_use]
    pub fn get_method(&self, index: usize) -> Option<&MethodInfo> {
        self.class_file.methods.get(index)
    }

    #[must_use]
    pub fn methods(&self) -> &[MethodInfo] {
        self.class_file.methods.as_slice()
    }

    #[must_use]
    pub fn access_flags(&self) -> ClassAccessFlags {
        self.class_file.access_flags
    }

    pub(crate) fn get_this_class_name(&self) -> Result<&str, ClassFileIndexError> {
        let this_class = self
            .get_t(self.class_file.this_class)
            .ok_or(ClassFileIndexError::InvalidThisClassIndex)?;
        self.get_t(this_class.name_index)
            .map(|x| x.utf8_string.as_str())
            .ok_or(ClassFileIndexError::InvalidThisClassNameIndex)
    }

    pub(crate) fn get_super_class_name(&self) -> Result<Option<&str>, ClassFileIndexError> {
        // There is no base class
        // Only java/lang/Object should have no base class, but we don't do that verification here
        if self.class_file.super_class.is_zero() {
            return Ok(None);
        }

        let super_class = self
            .get_t(self.class_file.super_class)
            .ok_or(ClassFileIndexError::InvalidSuperClassIndex)?;
        self.get_t(super_class.name_index)
            .map(|x| x.utf8_string.as_str())
            .map(Some)
            .ok_or(ClassFileIndexError::InvalidSuperClassNameIndex)
    }

    pub(crate) fn get_super_class_id(
        &self,
        class_names: &mut ClassNames,
    ) -> Result<Option<ClassFileId>, ClassFileIndexError> {
        Ok(self
            .get_super_class_name()?
            .map(|x| class_names.gcid_from_str(x)))
    }

    pub fn interfaces_indices_iter(
        &self,
    ) -> impl Iterator<Item = ConstantPoolIndexRaw<ClassConstant>> + '_ {
        self.class_file.interfaces.iter().copied()
    }
}

#[derive(Debug, Clone)]
pub enum ClassVariant {
    Class(Class),
    Array(ArrayClass),
}
impl ClassVariant {
    #[must_use]
    pub fn id(&self) -> ClassId {
        match self {
            Self::Class(x) => x.id,
            Self::Array(x) => x.id,
        }
    }

    #[must_use]
    pub fn super_id(&self) -> Option<ClassId> {
        match self {
            Self::Class(x) => x.super_id(),
            Self::Array(x) => Some(x.super_id()),
        }
    }

    #[must_use]
    pub fn access_flags(&self) -> ClassAccessFlags {
        match self {
            Self::Class(x) => x.access_flags,
            Self::Array(x) => x.access_flags,
        }
    }

    #[must_use]
    pub fn as_class(&self) -> Option<&Class> {
        match self {
            Self::Class(x) => Some(x),
            Self::Array(_) => None,
        }
    }

    #[must_use]
    pub fn as_array(&self) -> Option<&ArrayClass> {
        match self {
            Self::Class(_) => None,
            Self::Array(x) => Some(x),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Class {
    pub(crate) id: ClassId,
    pub(crate) super_class: Option<ClassFileId>,
    pub(crate) package: Option<PackageId>,
    pub(crate) access_flags: ClassAccessFlags,
    /// This is just the length of methods
    /// Not all methods are guaranteed to be initialized
    /// 0..last_method_id
    pub(crate) len_method_idx: MethodIndex,
}
impl Class {
    pub(crate) fn new(
        id: ClassId,
        super_class: Option<ClassFileId>,
        package: Option<PackageId>,
        access_flags: ClassAccessFlags,
        len_method_idx: MethodIndex,
    ) -> Self {
        Self {
            id,
            super_class,
            package,
            access_flags,
            len_method_idx,
        }
    }

    #[must_use]
    pub fn id(&self) -> ClassId {
        self.id
    }

    #[must_use]
    pub fn super_id(&self) -> Option<ClassFileId> {
        self.super_class
    }

    #[must_use]
    pub fn package(&self) -> Option<PackageId> {
        self.package
    }

    /// Iterate over all method ids that this method has.
    /// Note that this is just the ids, they are not guaranteed to be loaded.
    pub fn iter_method_ids(&self) -> impl Iterator<Item = MethodId> {
        let class_id = self.id;
        (0..self.len_method_idx).map(move |idx| MethodId::unchecked_compose(class_id, idx))
    }
}

// TODO: Are arrays in the same package as their defining type?
#[derive(Debug, Clone)]
pub struct ArrayClass {
    pub(crate) id: ClassId,
    pub(crate) name: String,
    pub(crate) component_type: ArrayComponentType,
    /// Always "java/lang/Object"
    pub(crate) super_class: ClassId,
    pub(crate) access_flags: ClassAccessFlags,
}
impl ArrayClass {
    // TODO: provide more libsound ways of creating this
    #[must_use]
    pub fn new_unchecked(
        id: ClassId,
        name: String,
        component_type: ArrayComponentType,
        super_class: ClassId,
        access_flags: ClassAccessFlags,
    ) -> Self {
        ArrayClass {
            id,
            name,
            component_type,
            super_class,
            access_flags,
        }
    }

    #[must_use]
    /// Note: This should not be used for strictly identifying
    /// This is strictly for debug purposes
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[must_use]
    pub fn id(&self) -> ClassId {
        self.id
    }

    #[must_use]
    pub fn component_type(&self) -> ArrayComponentType {
        self.component_type.clone()
    }

    #[must_use]
    pub fn super_id(&self) -> ClassId {
        self.super_class
    }
}

/// NOTE: We could have various other types, like unsigned versions, to allow for more granular type
/// checking, but that only makes sense if it can be determined.
#[derive(Debug, Clone)]
pub enum ArrayComponentType {
    Boolean,
    Char,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    Class(ClassId),
}
impl ArrayComponentType {
    #[must_use]
    pub fn is_primitive(&self) -> bool {
        !matches!(self, ArrayComponentType::Class(_))
    }

    #[must_use]
    /// Convert to class id if it is of the `Class` variant, aka if it is non-Primitive
    pub fn into_class_id(self) -> Option<ClassId> {
        match self {
            ArrayComponentType::Class(id) => Some(id),
            _ => None,
        }
    }

    pub fn to_desc_string(&self, class_names: &mut ClassNames) -> Result<String, BadIdError> {
        match self {
            ArrayComponentType::Byte => Ok("B".to_owned()),
            ArrayComponentType::Char => Ok("C".to_owned()),
            ArrayComponentType::Double => Ok("D".to_owned()),
            ArrayComponentType::Float => Ok("F".to_owned()),
            ArrayComponentType::Int => Ok("I".to_owned()),
            ArrayComponentType::Long => Ok("J".to_owned()),
            ArrayComponentType::Class(class_id) => {
                let name = class_names.name_from_gcid(*class_id)?;
                let path = name.path();
                if name.is_array() {
                    // If we have the id for an array then we just use the singular path it has
                    // because writing it as an object is incorrect.
                    Ok(path[0].clone())
                } else {
                    Ok(format!("L{path};", path = path.join("/")))
                }
            }
            ArrayComponentType::Short => Ok("S".to_owned()),
            ArrayComponentType::Boolean => Ok("Z".to_owned()),
        }
    }
}
// TODO: Make From<DescriptorTypeBasic>
impl From<PrimitiveType> for ArrayComponentType {
    fn from(prim: PrimitiveType) -> ArrayComponentType {
        match prim {
            PrimitiveType::Byte | PrimitiveType::UnsignedByte => ArrayComponentType::Byte,
            PrimitiveType::Short | PrimitiveType::UnsignedShort => ArrayComponentType::Short,
            PrimitiveType::Int => ArrayComponentType::Int,
            PrimitiveType::Long => ArrayComponentType::Long,
            PrimitiveType::Float => ArrayComponentType::Float,
            PrimitiveType::Double => ArrayComponentType::Double,
            PrimitiveType::Char => ArrayComponentType::Char,
            PrimitiveType::Boolean => ArrayComponentType::Boolean,
        }
    }
}
