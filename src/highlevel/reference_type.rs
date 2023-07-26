use std::ops::Deref;

use crate::{
    client::ClientError,
    jvm::{FieldModifiers, TypeModifiers},
    spec::{
        reference_type::{
            self, ClassFileVersion, ClassFileVersionReply, ConstantPool, ConstantPoolReply, Fields,
            FieldsWithGeneric, InstanceLimit, Instances, Interfaces, Modifiers, NestedTypes,
            Signature, SignatureWithGeneric, SignatureWithGenericReply, SourceDebugExtension,
            SourceFile, Status,
        },
        virtual_machine::InstanceCounts,
        ClassStatus, ReferenceTypeID, TaggedReferenceTypeID, TypeTag,
    },
};

use super::{
    ArrayType, ClassLoader, ClassObject, ClassType, InterfaceType, JvmObject, JvmReferenceField,
    PlainJvmObject, SharedClient, TaggedObject,
};

pub type ReferenceType = PlainJvmObject<ReferenceTypeID>;

impl ReferenceType {
    pub fn signature(&self) -> Result<String, ClientError> {
        self.client().get().send(Signature::new(self.id()))
    }

    pub fn class_loader(&self) -> Result<Option<ClassLoader>, ClientError> {
        let id = self
            .client()
            .get()
            .send(reference_type::ClassLoader::new(self.id()))?;
        let class_loader = id.map(|id| ClassLoader::new(self.client().clone(), id));
        Ok(class_loader)
    }

    pub fn modifiers(&self) -> Result<TypeModifiers, ClientError> {
        self.client().get().send(Modifiers::new(self.id()))
    }

    pub fn fields(&self) -> Result<Vec<StaticField>, ClientError> {
        let fields = self
            .client()
            .get()
            .send(Fields::new(self.id()))?
            .into_iter()
            .map(|f| StaticField {
                object: self.child(f.field_id),
                name: f.name,
                signature: f.signature,
                generic_signature: None,
                modifiers: f.mod_bits,
            })
            .collect();
        Ok(fields)
    }

    // todo: move this to ClassType/InterfaceType, dispatch here maybe?
    // pub fn methods(&self) -> Result<Vec<ReferenceMethod>, ClientError> {
    //     let methods = self
    //         .client()
    //         .get()
    //         .send(Methods::new(self.id()))?
    //         .into_iter()
    //         .map(|m| ReferenceMethod {
    //             object: self.child(m.method_id),
    //             name: m.name,
    //             signature: m.signature,
    //             generic_signature: None,
    //             modifiers: m.mod_bits,
    //         })
    //         .collect();
    //     Ok(methods)
    // }

    pub fn fields_generic(&self) -> Result<Vec<StaticField>, ClientError> {
        let fields = self
            .client()
            .get()
            .send(FieldsWithGeneric::new(self.id()))?
            .into_iter()
            .map(|f| StaticField {
                object: self.child(f.field_id),
                name: f.name,
                signature: f.signature,
                generic_signature: f.generic_signature,
                modifiers: f.mod_bits,
            })
            .collect();
        Ok(fields)
    }

    pub fn source_file(&self) -> Result<String, ClientError> {
        self.client().get().send(SourceFile::new(self.id()))
    }

    pub fn nested_types(&self) -> Result<Vec<TaggedReferenceType>, ClientError> {
        let types = self.client().get().send(NestedTypes::new(self.id()))?;
        let types = types
            .into_iter()
            .map(|id| TaggedReferenceType::new(self.client().clone(), id))
            .collect();
        Ok(types)
    }

    pub fn status(&self) -> Result<ClassStatus, ClientError> {
        self.client().get().send(Status::new(self.id()))
    }

    pub fn interfaces(&self) -> Result<Vec<InterfaceType>, ClientError> {
        let interfaces = self.client().get().send(Interfaces::new(self.id()))?;
        let interfaces = interfaces
            .into_iter()
            .map(|id| InterfaceType::new(self.client().clone(), id))
            .collect();
        Ok(interfaces)
    }

    pub fn class(&self) -> Result<ClassObject, ClientError> {
        let id = self
            .client()
            .get()
            .send(reference_type::ClassObject::new(self.id()))?;
        Ok(ClassObject::new(self.client().clone(), id))
    }

    pub fn source_debug_extension(&self) -> Result<String, ClientError> {
        self.client()
            .get()
            .send(SourceDebugExtension::new(self.id()))
    }

    pub fn signature_generic(&self) -> Result<SignatureWithGenericReply, ClientError> {
        self.client()
            .get()
            .send(SignatureWithGeneric::new(self.id()))
    }

    pub fn instances(&self, limit: InstanceLimit) -> Result<Vec<TaggedObject>, ClientError> {
        let instances = self.client().get().send(Instances::new(self.id(), limit))?;
        let instances = instances
            .into_iter()
            .map(|id| TaggedObject::new(self.client().clone(), id))
            .collect();
        Ok(instances)
    }

    pub fn class_file_version(&self) -> Result<ClassFileVersionReply, ClientError> {
        self.client().get().send(ClassFileVersion::new(self.id()))
    }

    pub fn constant_pool(&self) -> Result<ConstantPoolReply, ClientError> {
        self.client().get().send(ConstantPool::new(self.id()))
    }
}

impl ReferenceType {
    pub fn instance_count(&self) -> Result<u64, ClientError> {
        let [count] = self.client().get().send(InstanceCounts::new([self.id()]))?;
        Ok(count)
    }

    pub fn field(&self, name: &str) -> Result<StaticField, ClientError> {
        self.fields()?
            .into_iter()
            .find(|f| f.name == name)
            .ok_or_else(|| todo!("High-level errors")) // or just option?
                                                       // ergomomics..
    }

    pub fn field_generic(&self, name: &str) -> Result<StaticField, ClientError> {
        self.fields_generic()?
            .into_iter()
            .find(|f| f.name == name)
            .ok_or_else(|| todo!("High-level errors"))
    }
}

#[derive(Debug)]
pub struct StaticField {
    pub name: String,
    pub signature: String,
    pub generic_signature: Option<String>,
    pub modifiers: FieldModifiers,
    object: JvmReferenceField,
}

impl Deref for StaticField {
    type Target = JvmReferenceField;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

// #[derive(Debug)]
// pub struct ReferenceMethod {
//     pub name: String,
//     pub signature: String,
//     pub generic_signature: Option<String>,
//     pub modifiers: MethodModifiers,
//     object: JvmReferenceMethod,
// }

// impl Deref for ReferenceMethod {
//     type Target = JvmReferenceMethod;

//     fn deref(&self) -> &Self::Target {
//         &self.object
//     }
// }

#[derive(Debug)]
#[repr(u8)]
pub enum TaggedReferenceType {
    Class(ClassType) = TypeTag::Class as u8,
    Interface(InterfaceType) = TypeTag::Interface as u8,
    Array(ArrayType) = TypeTag::Array as u8,
}

impl TaggedReferenceType {
    pub fn new(client: impl Into<SharedClient>, id: TaggedReferenceTypeID) -> Self {
        use TaggedReferenceType::*;
        use TaggedReferenceTypeID as ID;

        match id {
            ID::Class(id) => Class(ClassType::new(client, id)),
            ID::Interface(id) => Interface(InterfaceType::new(client, id)),
            ID::Array(id) => Array(ArrayType::new(client, id)),
        }
    }

    pub fn tag(&self) -> TypeTag {
        // SAFETY: Self and TypeTag fulfill the requirements
        unsafe { crate::spec::tag(self) }
    }

    pub fn unwrap_class(self) -> ClassType {
        match self {
            TaggedReferenceType::Class(class) => class,
            _ => panic!("Expected a class"),
        }
    }

    pub fn unwrap_interface(self) -> InterfaceType {
        match self {
            TaggedReferenceType::Interface(interface) => interface,
            _ => panic!("Expected an interface"),
        }
    }

    pub fn unwrap_array(self) -> ArrayType {
        match self {
            TaggedReferenceType::Array(array) => array,
            _ => panic!("Expected an array"),
        }
    }
}

impl Deref for TaggedReferenceType {
    type Target = ReferenceType;

    fn deref(&self) -> &Self::Target {
        use TaggedReferenceType::*;

        match self {
            Class(ref_type) => ref_type,
            Interface(ref_type) => ref_type,
            Array(ref_type) => ref_type,
        }
    }
}
