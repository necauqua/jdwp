use jdwp::{
    highlevel::{JvmObject, ReferenceType, TaggedReferenceType, VM},
    jvm::{ConstantPoolItem, ConstantPoolValue, FieldModifiers},
    spec::{
        reference_type::{
            ClassFileVersion, ConstantPool, InstanceLimit, Methods, MethodsWithGeneric,
        },
        virtual_machine::ClassBySignature,
    },
};
use std::{assert_eq, error::Error, io::Cursor, ops::Deref};

mod common;

use common::Result;

const OUR_CLS: &str = "LBasic;";
const ARRAY_CLS: &str = "[I";

const CASES: &[&str] = &[OUR_CLS, "Ljava/lang/String;", "Ljava/util/List;", ARRAY_CLS];

trait VmExt {
    fn call_for_types<R, E: Error + 'static>(
        &self,
        signatures: &[&str],
        call: fn(TaggedReferenceType) -> std::result::Result<R, E>,
    ) -> Result<Vec<R>>;
}

impl VmExt for VM {
    fn call_for_types<R, E: Error + 'static>(
        &self,
        signatures: &[&str],
        call: fn(TaggedReferenceType) -> std::result::Result<R, E>,
    ) -> Result<Vec<R>> {
        signatures
            .iter()
            .map(|item| Ok(call(self.class_by_signature(item)?.0)?))
            .collect()
    }
}

trait CollExt {
    fn signatures(self) -> Result<Vec<String>>;
}

impl<I> CollExt for I
where
    I: IntoIterator,
    I::Item: Deref<Target = ReferenceType>,
{
    fn signatures(self) -> Result<Vec<String>> {
        let mut sigs = self
            .into_iter()
            .map(|ref_type| Ok(ref_type.signature()?))
            .collect::<Result<Vec<_>>>()?;
        sigs.sort_unstable();
        Ok(sigs)
    }
}

#[test]
fn signature() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let signatures = vm.call_for_types(CASES, |t| t.signature())?;

    assert_snapshot!(signatures, @r###"
    [
        "LBasic;",
        "Ljava/lang/String;",
        "Ljava/util/List;",
        "[I",
    ]"###);

    Ok(())
}

#[test]
fn signature_generic() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    // String has extra interfaces in java 17
    let signatures = vm.call_for_types(&[OUR_CLS, "Ljava/util/List;", ARRAY_CLS], |t| {
        t.signature_generic()
    })?;

    assert_snapshot!(signatures, @r###"
    [
        SignatureWithGenericReply {
            signature: "LBasic;",
            generic_signature: "<T:Ljava/lang/Object;>Ljava/lang/Object;Ljava/util/function/IntSupplier;",
        },
        SignatureWithGenericReply {
            signature: "Ljava/util/List;",
            generic_signature: "<E:Ljava/lang/Object;>Ljava/lang/Object;Ljava/util/Collection<TE;>;",
        },
        SignatureWithGenericReply {
            signature: "[I",
            generic_signature: "",
        },
    ]
    "###);

    Ok(())
}

#[test]
fn source_debug_extension() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let result = vm.class_by_signature(OUR_CLS)?.0.source_debug_extension();

    assert_snapshot!(result, @r###"
    Err(
        HostError(
            AbsentInformation,
        ),
    )
    "###);

    Ok(())
}

#[test]
fn class_loader() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let class_loaders = vm.call_for_types(CASES, |t| t.class_loader())?;

    assert_snapshot!(class_loaders, @r###"
    [
        Some(
            WrapperJvmObject(
                ClassLoaderID(opaque),
            ),
        ),
        None,
        None,
        None,
    ]
    "###);

    Ok(())
}

#[test]
fn visible_classes() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let (ref_type, _) = vm.class_by_signature(OUR_CLS)?;
    let class_loader = ref_type.class_loader()?.unwrap();

    let visible_classes = class_loader.visible_classes()?;

    let signatures = visible_classes
        .iter()
        .map(|c| Ok(c.signature()?))
        .collect::<Result<Vec<_>>>()?;

    const EXPECTED: &[&str] = &[
        OUR_CLS,
        "LBasic$NestedInterface;",
        "Ljava/lang/Class;",
        "Ljava/lang/ClassLoader;",
        "Ljava/lang/Thread;",
        "Ljava/lang/System;",
    ];

    assert!(
        signatures.iter().any(|s| EXPECTED.contains(&&**s)),
        "Visible classes don't contain our expected subset"
    );

    Ok(())
}

#[test]
fn modifiers() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let modifiers = vm.call_for_types(CASES, |t| t.modifiers())?;

    assert_snapshot!(modifiers, @r###"
    [
        TypeModifiers(
            SUPER,
        ),
        TypeModifiers(
            PUBLIC | FINAL | SUPER,
        ),
        TypeModifiers(
            PUBLIC | INTERFACE | ABSTRACT,
        ),
        TypeModifiers(
            PUBLIC | FINAL | ABSTRACT,
        ),
    ]
    "###);

    Ok(())
}

#[test]
fn fields() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;
    let (class_type, _) = vm.class_by_signature(OUR_CLS)?;
    let mut fields = class_type.fields()?;
    fields.sort_by_key(|f| f.name.clone());

    assert_snapshot!(fields, @r###"
    [
        StaticField {
            name: "running",
            signature: "LBasic;",
            generic_signature: None,
            modifiers: FieldModifiers(
                PUBLIC | STATIC | 0x800,
            ),
            object: NestedJvmObject(
                ReferenceTypeID(2),
                FieldID(opaque),
            ),
        },
        StaticField {
            name: "secondInstance",
            signature: "LBasic;",
            generic_signature: None,
            modifiers: FieldModifiers(
                PUBLIC | STATIC | 0x800,
            ),
            object: NestedJvmObject(
                ReferenceTypeID(2),
                FieldID(opaque),
            ),
        },
        StaticField {
            name: "staticInt",
            signature: "I",
            generic_signature: None,
            modifiers: FieldModifiers(
                STATIC,
            ),
            object: NestedJvmObject(
                ReferenceTypeID(2),
                FieldID(opaque),
            ),
        },
        StaticField {
            name: "ticks",
            signature: "J",
            generic_signature: None,
            modifiers: FieldModifiers(
                PUBLIC,
            ),
            object: NestedJvmObject(
                ReferenceTypeID(2),
                FieldID(opaque),
            ),
        },
        StaticField {
            name: "unused",
            signature: "Ljava/lang/String;",
            generic_signature: None,
            modifiers: FieldModifiers(
                FINAL,
            ),
            object: NestedJvmObject(
                ReferenceTypeID(2),
                FieldID(opaque),
            ),
        },
    ]
    "###);

    Ok(())
}

#[test]
fn fields_generic() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;
    let (class_type, _) = vm.class_by_signature(OUR_CLS)?;
    let mut fields = class_type.fields_generic()?;
    fields.sort_by_key(|f| f.name.clone());

    assert_snapshot!(fields, @r###"
    [
        StaticField {
            name: "running",
            signature: "LBasic;",
            generic_signature: Some(
                "LBasic<Ljava/lang/String;>;",
            ),
            modifiers: FieldModifiers(
                PUBLIC | STATIC | 0x800,
            ),
            object: NestedJvmObject(
                ReferenceTypeID(2),
                FieldID(opaque),
            ),
        },
        StaticField {
            name: "secondInstance",
            signature: "LBasic;",
            generic_signature: Some(
                "LBasic<*>;",
            ),
            modifiers: FieldModifiers(
                PUBLIC | STATIC | 0x800,
            ),
            object: NestedJvmObject(
                ReferenceTypeID(2),
                FieldID(opaque),
            ),
        },
        StaticField {
            name: "staticInt",
            signature: "I",
            generic_signature: None,
            modifiers: FieldModifiers(
                STATIC,
            ),
            object: NestedJvmObject(
                ReferenceTypeID(2),
                FieldID(opaque),
            ),
        },
        StaticField {
            name: "ticks",
            signature: "J",
            generic_signature: None,
            modifiers: FieldModifiers(
                PUBLIC,
            ),
            object: NestedJvmObject(
                ReferenceTypeID(2),
                FieldID(opaque),
            ),
        },
        StaticField {
            name: "unused",
            signature: "Ljava/lang/String;",
            generic_signature: None,
            modifiers: FieldModifiers(
                FINAL,
            ),
            object: NestedJvmObject(
                ReferenceTypeID(2),
                FieldID(opaque),
            ),
        },
    ]
    "###);

    Ok(())
}

#[test]
fn methods() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let (type_id, _) = *client.send(ClassBySignature::new(OUR_CLS))?;

    let mut methods = client.send(Methods::new(*type_id))?;
    methods.sort_by_key(|f| f.name.clone());

    assert_snapshot!(methods, @r###"
    [
        Method {
            method_id: MethodID(opaque),
            name: "<clinit>",
            signature: "()V",
            mod_bits: MethodModifiers(
                STATIC,
            ),
        },
        Method {
            method_id: MethodID(opaque),
            name: "<init>",
            signature: "()V",
            mod_bits: MethodModifiers(
                0x0,
            ),
        },
        Method {
            method_id: MethodID(opaque),
            name: "getAsInt",
            signature: "()I",
            mod_bits: MethodModifiers(
                PUBLIC,
            ),
        },
        Method {
            method_id: MethodID(opaque),
            name: "main",
            signature: "([Ljava/lang/String;)V",
            mod_bits: MethodModifiers(
                PUBLIC | STATIC,
            ),
        },
        Method {
            method_id: MethodID(opaque),
            name: "ping",
            signature: "(Ljava/lang/Object;)V",
            mod_bits: MethodModifiers(
                PRIVATE | STATIC,
            ),
        },
        Method {
            method_id: MethodID(opaque),
            name: "tick",
            signature: "()V",
            mod_bits: MethodModifiers(
                PUBLIC,
            ),
        },
        Method {
            method_id: MethodID(opaque),
            name: "withGeneric",
            signature: "(ILjava/util/function/IntSupplier;)V",
            mod_bits: MethodModifiers(
                STATIC,
            ),
        },
    ]
    "###);

    Ok(())
}

#[test]
fn methods_generic() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let (type_id, _) = *client.send(ClassBySignature::new(OUR_CLS))?;

    let mut methods = client.send(MethodsWithGeneric::new(*type_id))?;
    methods.sort_by_key(|f| f.name.clone());

    assert_snapshot!(methods, @r###"
    [
        MethodWithGeneric {
            method_id: MethodID(opaque),
            name: "<clinit>",
            signature: "()V",
            generic_signature: "",
            mod_bits: MethodModifiers(
                STATIC,
            ),
        },
        MethodWithGeneric {
            method_id: MethodID(opaque),
            name: "<init>",
            signature: "()V",
            generic_signature: "",
            mod_bits: MethodModifiers(
                0x0,
            ),
        },
        MethodWithGeneric {
            method_id: MethodID(opaque),
            name: "getAsInt",
            signature: "()I",
            generic_signature: "",
            mod_bits: MethodModifiers(
                PUBLIC,
            ),
        },
        MethodWithGeneric {
            method_id: MethodID(opaque),
            name: "main",
            signature: "([Ljava/lang/String;)V",
            generic_signature: "",
            mod_bits: MethodModifiers(
                PUBLIC | STATIC,
            ),
        },
        MethodWithGeneric {
            method_id: MethodID(opaque),
            name: "ping",
            signature: "(Ljava/lang/Object;)V",
            generic_signature: "",
            mod_bits: MethodModifiers(
                PRIVATE | STATIC,
            ),
        },
        MethodWithGeneric {
            method_id: MethodID(opaque),
            name: "tick",
            signature: "()V",
            generic_signature: "",
            mod_bits: MethodModifiers(
                PUBLIC,
            ),
        },
        MethodWithGeneric {
            method_id: MethodID(opaque),
            name: "withGeneric",
            signature: "(ILjava/util/function/IntSupplier;)V",
            generic_signature: "<T::Ljava/util/function/IntSupplier;>(ITT;)V",
            mod_bits: MethodModifiers(
                STATIC,
            ),
        },
    ]
    "###);

    Ok(())
}

#[test]
fn get_values() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;
    let (class_type, _) = vm.class_by_signature(OUR_CLS)?;
    let mut fields = class_type.fields()?;
    fields.sort_by_key(|f| f.name.clone());

    let fields = fields
        .into_iter()
        .filter_map(|f| {
            f.modifiers
                .contains(FieldModifiers::STATIC)
                .then_some(f.id())
        })
        .collect::<Vec<_>>();

    let values = class_type.child(fields).get()?;

    assert_snapshot!(values, @r###"
    [
        Object(
            ObjectID(opaque),
        ),
        Object(
            ObjectID(opaque),
        ),
        Int(
            42,
        ),
    ]
    "###);

    Ok(())
}

#[test]
fn source_file() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let source_files = vm
        .call_for_types(&[OUR_CLS, "Ljava/lang/String;", "Ljava/util/List;"], |t| {
            t.source_file()
        })?;

    assert_snapshot!(source_files, @r###"
    [
        "Basic.java",
        "String.java",
        "List.java",
    ]
    "###);

    let (array_type, _) = vm.class_by_signature(ARRAY_CLS)?;
    let array_source_file = array_type.source_file();

    assert_snapshot!(array_source_file, @r###"
    Err(
        HostError(
            AbsentInformation,
        ),
    )
    "###);

    Ok(())
}

#[test]
fn nested_types() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let (class_type, _) = vm.class_by_signature(OUR_CLS)?;

    let mut nested_types = class_type.nested_types()?;
    nested_types.sort_by_key(|t| t.tag() as u8);
    let nested_types = nested_types.signatures()?;

    assert_snapshot!(nested_types, @r###"
    [
        "LBasic$NestedClass;",
        "LBasic$NestedInterface;",
    ]
    "###);

    let (class_type, _) = vm.class_by_signature("Ljava/util/HashMap;")?;

    let mut nested_types = class_type.nested_types()?;
    nested_types.sort_by_key(|t| t.tag() as u8);
    let nested_types = nested_types.signatures()?;

    assert_snapshot!(nested_types, @r###"
    [
        "Ljava/util/HashMap$EntryIterator;",
        "Ljava/util/HashMap$EntrySet;",
        "Ljava/util/HashMap$EntrySpliterator;",
        "Ljava/util/HashMap$HashIterator;",
        "Ljava/util/HashMap$HashMapSpliterator;",
        "Ljava/util/HashMap$KeyIterator;",
        "Ljava/util/HashMap$KeySet;",
        "Ljava/util/HashMap$KeySpliterator;",
        "Ljava/util/HashMap$Node;",
        "Ljava/util/HashMap$TreeNode;",
        "Ljava/util/HashMap$UnsafeHolder;",
        "Ljava/util/HashMap$ValueIterator;",
        "Ljava/util/HashMap$ValueSpliterator;",
        "Ljava/util/HashMap$Values;",
    ]
    "###);

    Ok(())
}

#[test]
fn status() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let statuses = vm.call_for_types(CASES, |t| t.status())?;

    assert_snapshot!(statuses, @r###"
    [
        ClassStatus(
            VERIFIED | PREPARED | INITIALIZED,
        ),
        ClassStatus(
            VERIFIED | PREPARED | INITIALIZED,
        ),
        ClassStatus(
            VERIFIED | PREPARED | INITIALIZED,
        ),
        ClassStatus(
            0x0,
        ),
    ]
    "###);

    Ok(())
}

#[test]
fn interfaces() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let (class_type, _) = vm.class_by_signature(OUR_CLS)?;
    let interfaces = class_type.interfaces()?;
    let interfaces = interfaces.signatures()?;

    assert_snapshot!(interfaces, @r###"
    [
        "Ljava/util/function/IntSupplier;",
    ]
    "###);

    let (class_type, _) = vm.class_by_signature("Ljava/util/ArrayList;")?;
    let interfaces = class_type.interfaces()?;
    let interfaces = interfaces.signatures()?;

    assert_snapshot!(interfaces, @r###"
    [
        "Ljava/io/Serializable;",
        "Ljava/lang/Cloneable;",
        "Ljava/util/List;",
        "Ljava/util/RandomAccess;",
    ]
    "###);

    Ok(())
}

#[test]
fn class_object() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let (class_type, _) = vm.class_by_signature(OUR_CLS)?;
    let class = class_type.class()?;
    let class_type_2 = class.reflected_type()?;

    assert_eq!(class_type.id(), class_type_2.id());

    Ok(())
}

#[test]
fn instances() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let (class_type, _) = vm.class_by_signature(OUR_CLS)?;
    let instances = class_type.instances(InstanceLimit::limit(10))?;

    // the running instance and the one in the static field
    assert_snapshot!(instances, @r###"
    [
        Object(
            JvmObject(
                ObjectID(opaque),
            ),
        ),
        Object(
            JvmObject(
                ObjectID(opaque),
            ),
        ),
    ]
    "###);

    Ok(())
}

#[test]
fn class_file_version() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let (type_id, _) = *client.send(ClassBySignature::new(OUR_CLS))?;
    let version = client.send(ClassFileVersion::new(*type_id))?;

    let expected = match common::java_version() {
        8 => (52, 0),
        11 => (55, 0),
        17 => (61, 0),
        _ => {
            // ideally we'd mark this test as skipped
            println!("this test only works with java version 8, 11, or 17");
            return Ok(());
        }
    };

    assert_eq!((version.major_version, version.minor_version), expected);

    Ok(())
}

#[test]
fn superclass() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;
    let (class_type, _) = vm.class_by_signature(OUR_CLS)?;

    let superclass = class_type.unwrap_class().superclass()?.unwrap();
    let supersuperclass = superclass.superclass()?;

    assert_snapshot!(supersuperclass, @"None");

    let superclass = superclass.signature()?;
    assert_snapshot!(superclass, @r###""Ljava/lang/Object;""###);

    Ok(())
}

#[test]
fn constant_pool() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let (type_id, _) = *client.send(ClassBySignature::new(OUR_CLS))?;
    let constant_pool = client.send(ConstantPool::new(*type_id))?;
    let mut reader = Cursor::new(constant_pool.bytes);

    // pfew lol why did I bother so much
    let items = ConstantPoolItem::read_all(constant_pool.count, &mut reader)?;
    let values = ConstantPoolValue::resolve(&items)?;

    let mut values = values
        .into_iter()
        .filter_map(|v| match v {
            // NestMembers were introduced in java 11
            ConstantPoolValue::Utf8(s) if s.as_ref() == "NestMembers" => None,
            // for some reason java 8 doubles these - so we just ignore those lol, this is ugly
            ConstantPoolValue::Class(s)
                if [
                    "java/lang/InterruptedException",
                    "java/lang/ClassNotFoundException",
                ]
                .contains(&s.as_ref()) =>
            {
                None
            }
            _ => Some(format!("{:?}", v)),
        })
        .collect::<Vec<_>>();
    values.sort_unstable();

    assert_snapshot!(values, @r###"
    [
        "Class(\"Basic\")",
        "Class(\"Basic$NestedClass\")",
        "Class(\"Basic$NestedInterface\")",
        "Class(\"java/io/PrintStream\")",
        "Class(\"java/lang/Class\")",
        "Class(\"java/lang/Exception\")",
        "Class(\"java/lang/Object\")",
        "Class(\"java/lang/RuntimeException\")",
        "Class(\"java/lang/System\")",
        "Class(\"java/lang/Thread\")",
        "Class(\"java/util/HashMap\")",
        "Class(\"java/util/function/IntSupplier\")",
        "Fieldref(Ref { class: \"Basic\", name: \"running\", descriptor: \"LBasic;\" })",
        "Fieldref(Ref { class: \"Basic\", name: \"secondInstance\", descriptor: \"LBasic;\" })",
        "Fieldref(Ref { class: \"Basic\", name: \"staticInt\", descriptor: \"I\" })",
        "Fieldref(Ref { class: \"Basic\", name: \"ticks\", descriptor: \"J\" })",
        "Fieldref(Ref { class: \"Basic\", name: \"unused\", descriptor: \"Ljava/lang/String;\" })",
        "Fieldref(Ref { class: \"java/lang/System\", name: \"out\", descriptor: \"Ljava/io/PrintStream;\" })",
        "Long(500)",
        "Methodref(Ref { class: \"Basic\", name: \"<init>\", descriptor: \"()V\" })",
        "Methodref(Ref { class: \"Basic\", name: \"getAsInt\", descriptor: \"()I\" })",
        "Methodref(Ref { class: \"Basic\", name: \"ping\", descriptor: \"(Ljava/lang/Object;)V\" })",
        "Methodref(Ref { class: \"Basic\", name: \"tick\", descriptor: \"()V\" })",
        "Methodref(Ref { class: \"java/io/PrintStream\", name: \"println\", descriptor: \"(Ljava/lang/String;)V\" })",
        "Methodref(Ref { class: \"java/lang/Class\", name: \"forName\", descriptor: \"(Ljava/lang/String;)Ljava/lang/Class;\" })",
        "Methodref(Ref { class: \"java/lang/Class\", name: \"getClasses\", descriptor: \"()[Ljava/lang/Class;\" })",
        "Methodref(Ref { class: \"java/lang/Object\", name: \"<init>\", descriptor: \"()V\" })",
        "Methodref(Ref { class: \"java/lang/Object\", name: \"getClass\", descriptor: \"()Ljava/lang/Class;\" })",
        "Methodref(Ref { class: \"java/lang/RuntimeException\", name: \"<init>\", descriptor: \"(Ljava/lang/Throwable;)V\" })",
        "Methodref(Ref { class: \"java/lang/System\", name: \"exit\", descriptor: \"(I)V\" })",
        "Methodref(Ref { class: \"java/lang/Thread\", name: \"sleep\", descriptor: \"(J)V\" })",
        "NameAndType(NameAndType { name: \"<init>\", descriptor: \"()V\" })",
        "NameAndType(NameAndType { name: \"<init>\", descriptor: \"(Ljava/lang/Throwable;)V\" })",
        "NameAndType(NameAndType { name: \"exit\", descriptor: \"(I)V\" })",
        "NameAndType(NameAndType { name: \"forName\", descriptor: \"(Ljava/lang/String;)Ljava/lang/Class;\" })",
        "NameAndType(NameAndType { name: \"getAsInt\", descriptor: \"()I\" })",
        "NameAndType(NameAndType { name: \"getClass\", descriptor: \"()Ljava/lang/Class;\" })",
        "NameAndType(NameAndType { name: \"getClasses\", descriptor: \"()[Ljava/lang/Class;\" })",
        "NameAndType(NameAndType { name: \"out\", descriptor: \"Ljava/io/PrintStream;\" })",
        "NameAndType(NameAndType { name: \"ping\", descriptor: \"(Ljava/lang/Object;)V\" })",
        "NameAndType(NameAndType { name: \"println\", descriptor: \"(Ljava/lang/String;)V\" })",
        "NameAndType(NameAndType { name: \"running\", descriptor: \"LBasic;\" })",
        "NameAndType(NameAndType { name: \"secondInstance\", descriptor: \"LBasic;\" })",
        "NameAndType(NameAndType { name: \"sleep\", descriptor: \"(J)V\" })",
        "NameAndType(NameAndType { name: \"staticInt\", descriptor: \"I\" })",
        "NameAndType(NameAndType { name: \"tick\", descriptor: \"()V\" })",
        "NameAndType(NameAndType { name: \"ticks\", descriptor: \"J\" })",
        "NameAndType(NameAndType { name: \"unused\", descriptor: \"Ljava/lang/String;\" })",
        "String(\"Basic$NestedClass\")",
        "String(\"hello\")",
        "String(\"up\")",
        "Utf8(\"()I\")",
        "Utf8(\"()Ljava/lang/Class;\")",
        "Utf8(\"()V\")",
        "Utf8(\"()[Ljava/lang/Class;\")",
        "Utf8(\"(I)V\")",
        "Utf8(\"(ILjava/util/function/IntSupplier;)V\")",
        "Utf8(\"(J)V\")",
        "Utf8(\"(Ljava/lang/Object;)V\")",
        "Utf8(\"(Ljava/lang/String;)Ljava/lang/Class;\")",
        "Utf8(\"(Ljava/lang/String;)V\")",
        "Utf8(\"(Ljava/lang/Throwable;)V\")",
        "Utf8(\"([Ljava/lang/String;)V\")",
        "Utf8(\"<T::Ljava/util/function/IntSupplier;>(ITT;)V\")",
        "Utf8(\"<T:Ljava/lang/Object;>Ljava/lang/Object;Ljava/util/function/IntSupplier;\")",
        "Utf8(\"<clinit>\")",
        "Utf8(\"<init>\")",
        "Utf8(\"Basic\")",
        "Utf8(\"Basic$NestedClass\")",
        "Utf8(\"Basic$NestedInterface\")",
        "Utf8(\"Basic.java\")",
        "Utf8(\"Code\")",
        "Utf8(\"ConstantValue\")",
        "Utf8(\"Exceptions\")",
        "Utf8(\"I\")",
        "Utf8(\"InnerClasses\")",
        "Utf8(\"J\")",
        "Utf8(\"LBasic;\")",
        "Utf8(\"LBasic<*>;\")",
        "Utf8(\"LBasic<Ljava/lang/String;>;\")",
        "Utf8(\"LineNumberTable\")",
        "Utf8(\"Ljava/io/PrintStream;\")",
        "Utf8(\"Ljava/lang/String;\")",
        "Utf8(\"NestedClass\")",
        "Utf8(\"NestedInterface\")",
        "Utf8(\"Signature\")",
        "Utf8(\"SourceFile\")",
        "Utf8(\"StackMapTable\")",
        "Utf8(\"exit\")",
        "Utf8(\"forName\")",
        "Utf8(\"getAsInt\")",
        "Utf8(\"getClass\")",
        "Utf8(\"getClasses\")",
        "Utf8(\"hello\")",
        "Utf8(\"java/io/PrintStream\")",
        "Utf8(\"java/lang/Class\")",
        "Utf8(\"java/lang/ClassNotFoundException\")",
        "Utf8(\"java/lang/Exception\")",
        "Utf8(\"java/lang/InterruptedException\")",
        "Utf8(\"java/lang/Object\")",
        "Utf8(\"java/lang/RuntimeException\")",
        "Utf8(\"java/lang/System\")",
        "Utf8(\"java/lang/Thread\")",
        "Utf8(\"java/util/HashMap\")",
        "Utf8(\"java/util/function/IntSupplier\")",
        "Utf8(\"main\")",
        "Utf8(\"out\")",
        "Utf8(\"ping\")",
        "Utf8(\"println\")",
        "Utf8(\"running\")",
        "Utf8(\"secondInstance\")",
        "Utf8(\"sleep\")",
        "Utf8(\"staticInt\")",
        "Utf8(\"tick\")",
        "Utf8(\"ticks\")",
        "Utf8(\"unused\")",
        "Utf8(\"up\")",
        "Utf8(\"withGeneric\")",
    ]
    "###);

    Ok(())
}
