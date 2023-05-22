# jdwp
This is a generic JDWP protocol implementation.

It only provides the base types and encoding/decoding traits.

Also contains a ~~dumb~~ simple blocking JDWP client implementation.

Currently work in progress. All the commands from JDWP spec are implemented, but very little of them are tested.

Planned:
- Cover everything with tests - there are definitely human errors in this.
- More and better documentation (currently commands are repeating the JDWP spec)
- Async client?.

MSRV is 1.66.1

## License
This crate is licensed under the MIT license
