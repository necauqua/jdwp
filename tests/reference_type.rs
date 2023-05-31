use std::{assert_eq, fmt::Debug, io::Cursor};

use jdwp::{
    client::JdwpClient,
    codec::{JdwpReadable, JdwpWritable},
    commands::{
        class_object_reference::ReflectedType,
        reference_type::{
            ClassFileVersion, ClassLoader, ClassObject, ConstantPool, Fields, GetValues, Instances,
            Interfaces, Methods, Modifiers, NestedTypes, Signature, SourceFile, Status,
        },
        virtual_machine::ClassBySignature,
        Command,
    },
    jvm::{ConstantPoolItem, ConstantPoolValue, FieldModifiers},
    types::{InterfaceID, ReferenceTypeID, TaggedReferenceTypeID},
};

mod common;

use common::Result;

const OUR_CLS: &str = "LBasic;";
const ARRAY_CLS: &str = "[I";

const CASES: &[&str] = &[OUR_CLS, "Ljava/lang/String;", "Ljava/util/List;", ARRAY_CLS];

fn get_responses<C>(
    client: &mut JdwpClient,
    signatures: &[&str],
    new: fn(ReferenceTypeID) -> C,
) -> Result<Vec<C::Output>>
where
    C: Command + JdwpWritable + Debug,
    C::Output: JdwpReadable + Debug,
{
    signatures
        .iter()
        .map(|item| {
            let id = client.send(ClassBySignature::new(*item))?.type_id;
            Ok(client.send(new(*id))?)
        })
        .collect()
}

trait GetSignature {
    fn get_signature(&self, client: &mut JdwpClient) -> Result<String>;
}

impl GetSignature for TaggedReferenceTypeID {
    fn get_signature(&self, client: &mut JdwpClient) -> Result<String> {
        let sig = client.send(Signature::new(**self))?;
        Ok(format!("{:?}({sig})", self.tag()))
    }
}

impl GetSignature for InterfaceID {
    fn get_signature(&self, client: &mut JdwpClient) -> Result<String> {
        Ok(client.send(Signature::new(**self))?)
    }
}

fn get_signatures<I, S>(client: &mut JdwpClient, iterable: I) -> Result<Vec<String>>
where
    S: GetSignature,
    I: IntoIterator<Item = S>,
{
    let mut sigs = iterable
        .into_iter()
        .map(|ref_id| ref_id.get_signature(client))
        .collect::<Result<Vec<_>>>()?;
    sigs.sort_unstable();
    Ok(sigs)
}

#[test]
fn signature() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let signatures = get_responses(&mut client, CASES, Signature::new)?;

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
fn class_loader() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let class_loaders = get_responses(&mut client, CASES, ClassLoader::new)?;

    assert_snapshot!(class_loaders, @r###"
    [
        Some(
            [opaque_id],
        ),
        None,
        None,
        None,
    ]
    "###);

    Ok(())
}

#[test]
fn modifiers() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let modifiers = get_responses(&mut client, CASES, Modifiers::new)?;

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
    let mut client = common::launch_and_attach("basic")?;

    let id = client.send(ClassBySignature::new(OUR_CLS))?.type_id;

    let mut fields = client.send(Fields::new(*id))?;
    fields.sort_by_key(|f| f.name.clone());

    assert_snapshot!(fields, @r###"
    [
        Field {
            field_id: [opaque_id],
            name: "running",
            signature: "LBasic;",
            mod_bits: FieldModifiers(
                PUBLIC | STATIC,
            ),
        },
        Field {
            field_id: [opaque_id],
            name: "secondInstance",
            signature: "LBasic;",
            mod_bits: FieldModifiers(
                PUBLIC | STATIC,
            ),
        },
        Field {
            field_id: [opaque_id],
            name: "staticInt",
            signature: "I",
            mod_bits: FieldModifiers(
                STATIC,
            ),
        },
        Field {
            field_id: [opaque_id],
            name: "ticks",
            signature: "J",
            mod_bits: FieldModifiers(
                PUBLIC,
            ),
        },
        Field {
            field_id: [opaque_id],
            name: "unused",
            signature: "Ljava/lang/String;",
            mod_bits: FieldModifiers(
                FINAL,
            ),
        },
    ]
    "###);

    Ok(())
}

#[test]
fn methods() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let id = client.send(ClassBySignature::new(OUR_CLS))?.type_id;

    let mut methods = client.send(Methods::new(*id))?;
    methods.sort_by_key(|f| f.name.clone());

    assert_snapshot!(methods, @r###"
    [
        Method {
            method_id: [opaque_id],
            name: "<clinit>",
            signature: "()V",
            mod_bits: MethodModifiers(
                STATIC,
            ),
        },
        Method {
            method_id: [opaque_id],
            name: "<init>",
            signature: "()V",
            mod_bits: MethodModifiers(
                0x0,
            ),
        },
        Method {
            method_id: [opaque_id],
            name: "getAsInt",
            signature: "()I",
            mod_bits: MethodModifiers(
                PUBLIC,
            ),
        },
        Method {
            method_id: [opaque_id],
            name: "main",
            signature: "([Ljava/lang/String;)V",
            mod_bits: MethodModifiers(
                PUBLIC | STATIC,
            ),
        },
        Method {
            method_id: [opaque_id],
            name: "ping",
            signature: "(Ljava/lang/Object;)V",
            mod_bits: MethodModifiers(
                PRIVATE | STATIC,
            ),
        },
        Method {
            method_id: [opaque_id],
            name: "tick",
            signature: "()V",
            mod_bits: MethodModifiers(
                PUBLIC,
            ),
        },
    ]
    "###);

    Ok(())
}

#[test]
fn get_values() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let id = client.send(ClassBySignature::new(OUR_CLS))?.type_id;

    let mut fields = client.send(Fields::new(*id))?;
    fields.sort_by_key(|f| f.name.clone());

    let fields = fields
        .into_iter()
        .filter_map(|f| {
            f.mod_bits
                .contains(FieldModifiers::STATIC)
                .then_some(f.field_id)
        })
        .collect::<Vec<_>>();

    let values = client.send(GetValues::new(*id, fields))?;

    assert_snapshot!(values, @r###"
    [
        Object(
            [opaque_id],
        ),
        Object(
            [opaque_id],
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
    let mut client = common::launch_and_attach("basic")?;

    let source_files = get_responses(
        &mut client,
        &[OUR_CLS, "Ljava/lang/String;", "Ljava/util/List;"],
        SourceFile::new,
    )?;

    assert_snapshot!(source_files, @r###"
    [
        "Basic.java",
        "String.java",
        "List.java",
    ]
    "###);

    let id = client.send(ClassBySignature::new(ARRAY_CLS))?.type_id;
    let array_source_file = client.send(SourceFile::new(*id));

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
    let mut client = common::launch_and_attach("basic")?;

    let id = client.send(ClassBySignature::new(OUR_CLS))?.type_id;

    let mut nested_types = client.send(NestedTypes::new(*id))?;
    nested_types.sort_by_key(|t| t.tag() as u8);

    let nested_types = get_signatures(&mut client, nested_types)?;

    assert_snapshot!(nested_types, @r###"
    [
        "Class(LBasic$NestedClass;)",
        "Interface(LBasic$NestedInterface;)",
    ]
    "###);

    let id = client
        .send(ClassBySignature::new("Ljava/util/HashMap;"))?
        .type_id;
    let mut nested_types = client.send(NestedTypes::new(*id))?;
    nested_types.sort_by_key(|t| t.tag() as u8);

    let nested_types = get_signatures(&mut client, nested_types)?;

    assert_snapshot!(nested_types, @r###"
    [
        "Class(Ljava/util/HashMap$EntryIterator;)",
        "Class(Ljava/util/HashMap$EntrySet;)",
        "Class(Ljava/util/HashMap$EntrySpliterator;)",
        "Class(Ljava/util/HashMap$HashIterator;)",
        "Class(Ljava/util/HashMap$HashMapSpliterator;)",
        "Class(Ljava/util/HashMap$KeyIterator;)",
        "Class(Ljava/util/HashMap$KeySet;)",
        "Class(Ljava/util/HashMap$KeySpliterator;)",
        "Class(Ljava/util/HashMap$Node;)",
        "Class(Ljava/util/HashMap$TreeNode;)",
        "Class(Ljava/util/HashMap$UnsafeHolder;)",
        "Class(Ljava/util/HashMap$ValueIterator;)",
        "Class(Ljava/util/HashMap$ValueSpliterator;)",
        "Class(Ljava/util/HashMap$Values;)",
    ]
    "###);

    Ok(())
}

#[test]
fn status() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let statuses = get_responses(&mut client, CASES, Status::new)?;

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
    let mut client = common::launch_and_attach("basic")?;

    let id = client.send(ClassBySignature::new(OUR_CLS))?.type_id;
    let interfaces = client.send(Interfaces::new(*id))?;
    let interfaces = get_signatures(&mut client, interfaces)?;

    assert_snapshot!(interfaces, @r###"
    [
        "Ljava/util/function/IntSupplier;",
    ]
    "###);

    let id = client
        .send(ClassBySignature::new("Ljava/util/ArrayList;"))?
        .type_id;
    let interfaces = client.send(Interfaces::new(*id))?;
    let interfaces = get_signatures(&mut client, interfaces)?;

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
    let mut client = common::launch_and_attach("basic")?;

    let id = client.send(ClassBySignature::new(OUR_CLS))?.type_id;
    let class_object = client.send(ClassObject::new(*id))?;
    let ref_id = client.send(ReflectedType::new(class_object))?;

    assert_eq!(id, ref_id);

    Ok(())
}

#[test]
fn instances() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let id = client.send(ClassBySignature::new(OUR_CLS))?.type_id;
    let instances = client.send(Instances::new(*id, 10))?;

    // the running instance and the one in the static field
    assert_snapshot!(instances, @r###"
    [
        Object(
            [opaque_id],
        ),
        Object(
            [opaque_id],
        ),
    ]
    "###);

    Ok(())
}

#[test]
fn class_file_version() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let id = client.send(ClassBySignature::new(OUR_CLS))?.type_id;
    let version = client.send(ClassFileVersion::new(*id))?;

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
fn constant_pool() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let id = client.send(ClassBySignature::new(OUR_CLS))?.type_id;
    let constant_pool = client.send(ConstantPool::new(*id))?;
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
        "Utf8(\"(J)V\")",
        "Utf8(\"(Ljava/lang/Object;)V\")",
        "Utf8(\"(Ljava/lang/String;)Ljava/lang/Class;\")",
        "Utf8(\"(Ljava/lang/String;)V\")",
        "Utf8(\"(Ljava/lang/Throwable;)V\")",
        "Utf8(\"([Ljava/lang/String;)V\")",
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
        "Utf8(\"LineNumberTable\")",
        "Utf8(\"Ljava/io/PrintStream;\")",
        "Utf8(\"Ljava/lang/String;\")",
        "Utf8(\"NestedClass\")",
        "Utf8(\"NestedInterface\")",
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
    ]
    "###);

    Ok(())
}
