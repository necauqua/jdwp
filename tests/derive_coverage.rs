use insta::assert_snapshot;
use jdwp::{
    enums::{EventKind, InvokeOptions, SuspendPolicy, Tag},
    event_modifier::Modifier,
    types::{JdwpId, TaggedObjectID, Value},
};

macro_rules! debug_and_clone {
    ($($p:ident::{$($e:expr,)*},)*) => {{
        let mut s = String::new();
        $(
            s.push_str(stringify!($p));
            s.push_str("::{\n");
            $(
                s.push_str(&format!("    {:?}\n", {
                    use jdwp::commands::$p::*;
                    $e.clone()
                }));
            )*
            s.push_str("}\n");
        )*
        s
    }}
}

fn id<T>() -> T
where
    T: JdwpId,
    T::Raw: From<u16>,
    // u16 fits in both u64 and i32, and so has an errorless Into impl for them, lol
{
    T::from_raw(123.into())
}

#[test]
fn manually_cover_clone_and_debug() {
    let all_commands = debug_and_clone![
        virtual_machine::{
            Version,
            ClassBySignature::new("Ljava/lang/Object;"),
            ClassesBySignatureStatic::<2>::new("Ljava/lang/Object;"),
            ClassesBySignature::new("Ljava/lang/Object;"),
            AllClasses,
            AllThreads,
            TopLevelThreadGroups,
            Dispose,
            IDSizes,
            Suspend,
            Resume,
            Exit::new(0),
            CreateString::new("Hello, world!"),
            Capabilities,
            ClassPaths,
            DisposeObjects::new(&[(id(), 123)]),
            HoldEvents,
            ReleaseEvents,
            CapabilitiesNew,
            RedefineClasses::new(&[(id(), vec![1, 2, 3, 123])]),
            SetDefaultStratum::new("NotJava"),
            AllClassesWithGeneric,
            InstanceCounts::new([id()]),
        },
        reference_type::{
            Signature::new(id()),
            ClassLoader::new(id()),
            Modifiers::new(id()),
            Fields::new(id()),
            Methods::new(id()),
            GetValues::new(id(), vec![id(), id()]),
            GetValues::new(id(), [id(), id()]),
            SourceFile::new(id()),
            NestedTypes::new(id()),
            Status::new(id()),
            Interfaces::new(id()),
            ClassObject::new(id()),
            SourceDebugExtension::new(id()),
            SignatureWithGeneric::new(id()),
            FieldsWithGeneric::new(id()),
            MethodsWithGeneric::new(id()),
            Instances::new(id(), 10),
            ClassFileVersion::new(id()),
            ConstantPool::new(id()),
        },
        class_type::{
            Superclass::new(id()),
            SetValues::new(id(), &[(id(), Value::Int(123).into()), (id(), Value::Object(id()).into())]),
            InvokeMethod::new(id(), id(), id(), &[Value::Int(123), Value::Object(id())], InvokeOptions::NONE),
            NewInstance::new(id(), id(), id(), &[Value::Int(123), Value::Object(id())], InvokeOptions::SINGLE_THREADED),
        },
        array_type::{
            NewInstance::new(id(), 10),
        },
        interface_type::{
            InvokeMethod::new(id(), id(), id(), &[Value::Int(123), Value::Object(id())], InvokeOptions::NONE),
        },
        method::{
            LineTable::new(id(), id()),
            VariableTable::new(id(), id()),
            Bytecodes::new(id(), id()),
            IsObsolete::new(id(), id()),
            VariableTableWithGeneric::new(id(), id()),
        },
        object_reference::{
            ReferenceType::new(id()),
            GetValues::new(id(), vec![id(), id()]),
            SetValues::new(id(), &[(id(), Value::Int(123).into()), (id(), Value::Object(id()).into())]),
            MonitorInfo::new(id()),
            InvokeMethod::new(id(), id(), id(), id(), &[Value::Int(123), Value::Object(id())], InvokeOptions::NONE),
            DisableCollection::new(id()),
            EnableCollection::new(id()),
            IsCollected::new(id()),
            ReferringObjects::new(id(), 10),
        },
        string_reference::{
            Value::new(id()),
        },
        thread_reference::{
            Name::new(id()),
            Suspend::new(id()),
            Resume::new(id()),
            Status::new(id()),
            ThreadGroup::new(id()),
            Frames::new(id(), 0, FrameLimit::AllRemaining),
            Frames::new(id(), 2, FrameLimit::Limit(10)),
            FrameCount::new(id()),
            OwnedMonitors::new(id()),
            CurrentContendedMonitor::new(id()),
            Stop::new(id(), TaggedObjectID::Object(id())),
            Interrupt::new(id()),
            SuspendCount::new(id()),
            OwnedMonitorsStackDepthInfo::new(id()),
            ForceEarlyReturn::new(id(), Value::Boolean(false)),
        },
        thread_group_reference::{
            Name::new(id()),
            Parent::new(id()),
            Children::new(id()),
        },
        array_reference::{
            Length::new(id()),
            GetValues::new(id(), 0, 10),
            SetValues::new(id(), 0, &[Value::Float(123.0).into(), Value::Float(42.0).into()]),
        },
        class_loader_reference::{
            VisibleClasses::new(id()),
        },
        event_request::{
            Set::new(EventKind::SingleStep, SuspendPolicy::All, &[Modifier::Count(10), Modifier::ThreadOnly(id())]),
            Clear::new(EventKind::Breakpoint, id()),
            ClearAllBreakpoints,
        },
        stack_frame::{
            GetValues::new(id(), id(), vec![(1, Tag::Object), (2, Tag::Int)]),
            GetValues::new(id(), id(), [(1, Tag::Object), (2, Tag::Int)]),
            SetValues::new(id(), id(), &[(1, Value::Int(123)), (2, Value::Object(id()))]),
            ThisObject::new(id(), id()),
            PopFrames::new(id(), id()),
        },
        class_object_reference::{
            ReflectedType::new(id()),
        },
        event::{
            Composite::new(SuspendPolicy::EventThread, vec![Event::VmStart(Some(id()), id())]),
        },
    ];

    assert_snapshot!(all_commands, @r###"
    virtual_machine::{
        Version
        ClassBySignature { signature: "Ljava/lang/Object;" }
        ClassesBySignatureStatic<2> { signature: "Ljava/lang/Object;" }
        ClassesBySignature { signature: "Ljava/lang/Object;" }
        AllClasses
        AllThreads
        TopLevelThreadGroups
        Dispose
        IDSizes
        Suspend
        Resume
        Exit { exit_code: 0 }
        CreateString { string: "Hello, world!" }
        Capabilities
        ClassPaths
        DisposeObjects { requests: [(ObjectID(123), 123)] }
        HoldEvents
        ReleaseEvents
        CapabilitiesNew
        RedefineClasses { classes: [(ReferenceTypeID(123), [1, 2, 3, 123])] }
        SetDefaultStratum { stratum_id: "NotJava" }
        AllClassesWithGeneric
        InstanceCounts { ref_types: [ReferenceTypeID(123)] }
    }
    reference_type::{
        Signature { ref_type: ReferenceTypeID(123) }
        ClassLoader { ref_type: ReferenceTypeID(123) }
        Modifiers { ref_type: ReferenceTypeID(123) }
        Fields { ref_type: ReferenceTypeID(123) }
        Methods { ref_type: ReferenceTypeID(123) }
        GetValues { ref_type: ReferenceTypeID(123), fields: [FieldID(123), FieldID(123)] }
        GetValues { ref_type: ReferenceTypeID(123), fields: [FieldID(123), FieldID(123)] }
        SourceFile { ref_type: ReferenceTypeID(123) }
        NestedTypes { ref_type: ReferenceTypeID(123) }
        Status { ref_type: ReferenceTypeID(123) }
        Interfaces { ref_type: ReferenceTypeID(123) }
        ClassObject { ref_type: ReferenceTypeID(123) }
        SourceDebugExtension { ref_type: ReferenceTypeID(123) }
        SignatureWithGeneric { ref_type: ReferenceTypeID(123) }
        FieldsWithGeneric { ref_type: ReferenceTypeID(123) }
        MethodsWithGeneric { ref_type: ReferenceTypeID(123) }
        Instances { ref_type: ReferenceTypeID(123), max_instances: 10 }
        ClassFileVersion { ref_type: ReferenceTypeID(123) }
        ConstantPool { ref_type: ReferenceTypeID(123) }
    }
    class_type::{
        Superclass { class_id: ClassID(123) }
        SetValues { class_id: ClassID(123), values: [(FieldID(123), UntaggedValue(Int(123))), (FieldID(123), UntaggedValue(Object(ObjectID(123))))] }
        InvokeMethod { class_id: ClassID(123), thread_id: ThreadID(123), method_id: MethodID(123), arguments: [Int(123), Object(ObjectID(123))], options: InvokeOptions(0x0) }
        NewInstance { class_id: ClassID(123), thread_id: ThreadID(123), method_id: MethodID(123), arguments: [Int(123), Object(ObjectID(123))], options: InvokeOptions(NONE | SINGLE_THREADED) }
    }
    array_type::{
        NewInstance { array_type_id: ArrayTypeID(123), length: 10 }
    }
    interface_type::{
        InvokeMethod { interface_id: InterfaceID(123), thread_id: ThreadID(123), method_id: MethodID(123), arguments: [Int(123), Object(ObjectID(123))], options: InvokeOptions(0x0) }
    }
    method::{
        LineTable { reference_type_id: ReferenceTypeID(123), method_id: MethodID(123) }
        VariableTable { reference_type_id: ReferenceTypeID(123), method_id: MethodID(123) }
        Bytecodes { reference_type_id: ReferenceTypeID(123), method_id: MethodID(123) }
        IsObsolete { reference_type_id: ReferenceTypeID(123), method_id: MethodID(123) }
        VariableTableWithGeneric { reference_type_id: ReferenceTypeID(123), method_id: MethodID(123) }
    }
    object_reference::{
        ReferenceType { object: ObjectID(123) }
        GetValues { object: ObjectID(123), fields: [FieldID(123), FieldID(123)] }
        SetValues { object: ObjectID(123), fields: [(FieldID(123), UntaggedValue(Int(123))), (FieldID(123), UntaggedValue(Object(ObjectID(123))))] }
        MonitorInfo { object: ObjectID(123) }
        InvokeMethod { object: ObjectID(123), thread: ThreadID(123), class: ClassID(123), method: FieldID(123), arguments: [Int(123), Object(ObjectID(123))], options: InvokeOptions(0x0) }
        DisableCollection { object: ObjectID(123) }
        EnableCollection { object: ObjectID(123) }
        IsCollected { object: ObjectID(123) }
        ReferringObjects { object: ObjectID(123), max_referrers: 10 }
    }
    string_reference::{
        Value { string_object: ObjectID(123) }
    }
    thread_reference::{
        Name { thread: ThreadID(123) }
        Suspend { thread: ThreadID(123) }
        Resume { thread: ThreadID(123) }
        Status { thread: ThreadID(123) }
        ThreadGroup { thread: ThreadID(123) }
        Frames { thread: ThreadID(123), start_frame: 0, limit: AllRemaining }
        Frames { thread: ThreadID(123), start_frame: 2, limit: Limit(10) }
        FrameCount { thread: ThreadID(123) }
        OwnedMonitors { thread: ThreadID(123) }
        CurrentContendedMonitor { thread: ThreadID(123) }
        Stop { thread: ThreadID(123), throwable: Object(ObjectID(123)) }
        Interrupt { thread: ThreadID(123) }
        SuspendCount { thread: ThreadID(123) }
        OwnedMonitorsStackDepthInfo { thread: ThreadID(123) }
        ForceEarlyReturn { thread: ThreadID(123), value: Boolean(false) }
    }
    thread_group_reference::{
        Name { group: ThreadGroupID(123) }
        Parent { group: ThreadGroupID(123) }
        Children { group: ThreadGroupID(123) }
    }
    array_reference::{
        Length { array_id: ArrayID(123) }
        GetValues { array_id: ArrayID(123), first_index: 0, length: 10 }
        SetValues { array_id: ArrayID(123), first_index: 0, values: [UntaggedValue(Float(123.0)), UntaggedValue(Float(42.0))] }
    }
    class_loader_reference::{
        VisibleClasses { class_loader_id: ClassLoaderID(123) }
    }
    event_request::{
        Set { event_kind: SingleStep, suspend_policy: All, modifiers: [Count(10), ThreadOnly(ThreadID(123))] }
        Clear { event_kind: Breakpoint, request_id: RequestID(123) }
        ClearAllBreakpoints
    }
    stack_frame::{
        GetValues { thread_id: ThreadID(123), frame_id: FrameID(123), slots: [(1, Object), (2, Int)] }
        GetValues { thread_id: ThreadID(123), frame_id: FrameID(123), slots: [(1, Object), (2, Int)] }
        SetValues { thread_id: ThreadID(123), frame_id: FrameID(123), slots: [(1, Int(123)), (2, Object(ObjectID(123)))] }
        ThisObject { thread_id: ThreadID(123), frame_id: FrameID(123) }
        PopFrames { thread_id: ThreadID(123), frame_id: FrameID(123) }
    }
    class_object_reference::{
        ReflectedType { class_object_id: ClassObjectID(123) }
    }
    event::{
        Composite { suspend_policy: EventThread, events: [VmStart(Some(RequestID(123)), ThreadID(123))] }
    }
    "###)
}
