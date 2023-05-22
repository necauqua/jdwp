# jdwp
This is a generic JDWP protocol implementation.

It only provides the base types and encoding/decoding traits.

Also contains a ~~dumb~~ simple blocking JDWP client implementation.

Currently work in progress.

Planned:
- Implement all the commands (currently ~half)
- Cover everything with tests (currently very little, but the setup is there) - there are definitely human errors in this.
- Async client?.

MSRV is 1.66.1

### JDWP commands implemented:
- [x] VirtualMachine Command Set (1)
  - [x] Version (1)
  - [x] ClassesBySignature (2)
  - [x] AllClasses (3)
  - [x] AllThreads (4)
  - [x] TopLevelThreadGroups (5)
  - [x] Dispose (6)
  - [x] IDSizes (7)
  - [x] Suspend (8)
  - [x] Resume (9)
  - [x] Exit (10)
  - [x] CreateString (11)
  - [x] Capabilities (12)
  - [x] ClassPaths (13)
  - [x] DisposeObjects (14)
  - [x] HoldEvents (15)
  - [x] ReleaseEvents (16)
  - [x] CapabilitiesNew (17)
  - [x] RedefineClasses (18)
  - [x] SetDefaultStratum (19)
  - [x] AllClassesWithGeneric (20)
  - [x] InstanceCounts (21)

- [x] ReferenceType Command Set (2)
  - [x] Signature (1)
  - [x] ClassLoader (2)
  - [x] Modifiers (3)
  - [x] Fields (4)
  - [x] Methods (5)
  - [x] GetValues (6)
  - [x] SourceFile (7)
  - [x] NestedTypes (8)
  - [x] Status (9)
  - [x] Interfaces (10)
  - [x] ClassObject (11)
  - [x] SourceDebugExtension (12)
  - [x] SignatureWithGeneric (13)
  - [x] FieldsWithGeneric (14)
  - [x] MethodsWithGeneric (15)
  - [x] Instances (16)
  - [x] ClassFileVersion (17)
  - [x] ConstantPool (18)

- [ ] ClassType Command Set (3)
  - [ ] Superclass (1)
  - [ ] SetValues (2)
  - [ ] InvokeMethod (3)
  - [ ] NewInstance (4)

- [x] ArrayType Command Set (4)
  - [x] NewInstance (1)

- [x] InterfaceType Command Set (5)

- [ ] Method Command Set (6)
  - [ ] LineTable (1)
  - [ ] VariableTable (2)
  - [ ] Bytecodes (3)
  - [ ] IsObsolete (4)
  - [ ] VariableTableWithGeneric (5)

- [x] Field Command Set (8)

- [ ] ObjectReference Command Set (9)
  - [ ] ReferenceType (1)
  - [ ] GetValues (2)
  - [ ] SetValues (3)
  - [ ] MonitorInfo (5)
  - [ ] InvokeMethod (6)
  - [ ] DisableCollection (7)
  - [ ] EnableCollection (8)
  - [ ] IsCollected (9)
  - [ ] ReferringObjects (10)

- [x] StringReference Command Set (10)
  - [x] Value (1)

- [ ] ThreadReference Command Set (11)
  - [x] Name (1)
  - [x] Suspend (2)
  - [ ] Resume (3)
  - [ ] Status (4)
  - [ ] ThreadGroup (5)
  - [ ] Frames (6)
  - [ ] FrameCount (7)
  - [ ] OwnedMonitors (8)
  - [ ] CurrentContendedMonitor (9)
  - [ ] Stop (10)
  - [ ] Interrupt (11)
  - [ ] SuspendCount (12)
  - [ ] OwnedMonitorsStackDepthInfo (13)
  - [ ] ForceEarlyReturn (14)

- [x] ThreadGroupReference Command Set (12)
  - [x] Name (1)
  - [x] Parent (2)
  - [x] Children (3)

- [x] ArrayReference Command Set (13)
  - [x] Length (1)
  - [x] GetValues (2)
  - [x] SetValues (3)

- [x] ClassLoaderReference Command Set (14)
  - [x] VisibleClasses (1)

- [x] EventRequest Command Set (15)
  - [x] Set (1)
  - [x] Clear (2)
  - [x] ClearAllBreakpoints (3)

- [ ] StackFrame Command Set (16)
  - [ ] GetValues (1)
  - [ ] SetValues (2)
  - [ ] ThisObject (3)
  - [ ] PopFrames (4)

- [x] ClassObjectReference Command Set (17)
  - [x] ReflectedType (1)

- [x] Event Command Set (64)
  - [x] Composite (100)

## License
This crate is licensed under the MIT license
