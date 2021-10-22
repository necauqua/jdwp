pub mod virtual_machine;

use crate::{
    codec::{JdwpReadable, JdwpWritable},
    CommandId,
};

pub(self) use jdwp_macros::jdwp_command;

pub trait Command: JdwpWritable {
    const ID: CommandId;
    type Output: JdwpReadable;
}

// VirtualMachine is done, pfew

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ReferenceType {
    Signature = 1,
    ClassLoader = 2,
    Modifiers = 3,
    Fields = 4,
    Methods = 5,
    GetValues = 6,
    SourceFile = 7,
    NestedTypes = 8,
    Status = 9,
    Interfaces = 10,
    ClassObject = 11,
    SourceDebugExtension = 12,
    SignatureWithGeneric = 13,
    FieldsWithGeneric = 14,
    MethodsWithGeneric = 15,
    Instances = 16,
    ClassFileVersion = 17,
    ConstantPool = 18,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ClassType {
    Superclass = 1,
    SetValues = 2,
    InvokeMethod = 3,
    NewInstance = 4,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ArrayType {
    NewInstance = 1,
}

// #[repr(u8)]
#[derive(Copy, Clone)]
pub enum InterfaceType {}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Method {
    LineTable = 1,
    VariableTable = 2,
    Bytecodes = 3,
    IsObsolete = 4,
    VariableTableWithGeneric = 5,
}

// #[repr(u8)]
#[derive(Copy, Clone)]
pub enum Field {}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ObjectReference {
    ReferenceType = 1,
    GetValues = 2,
    SetValues = 3,
    MonitorInfo = 5,
    InvokeMethod = 6,
    DisableCollection = 7,
    EnableCollection = 8,
    IsCollected = 9,
    ReferringObjects = 10,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum StringReference {
    Value = 1,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ThreadReference {
    Name = 1,
    Suspend = 2,
    Resume = 3,
    Status = 4,
    ThreadGroup = 5,
    Frames = 6,
    FrameCount = 7,
    OwnedMonitors = 8,
    CurrentContendedMonitor = 9,
    Stop = 10,
    Interrupt = 11,
    SuspendCount = 12,
    OwnedMonitorsStackDepthInfo = 13,
    ForceEarlyReturn = 14,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ThreadGroupReference {
    Name = 1,
    Parent = 2,
    Children = 3,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ArrayReference {
    Length = 1,
    GetValues = 2,
    SetValues = 3,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ClassLoaderReference {
    VisibleClasses = 1,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum EventRequest {
    Set = 1,
    Clear = 2,
    ClearAllBreakpoints = 3,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum StackFrame {
    GetValues = 1,
    SetValues = 2,
    ThisObject = 3,
    PopFrames = 4,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ClassObjectReference {
    ReflectedType = 1,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Event {
    Composite = 100,
}
