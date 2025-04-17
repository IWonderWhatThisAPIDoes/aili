# Aili-GDBState

Implementation of the [Program State model](../model) that relies
on the [GNU Project Debugger](https://www.sourceware.org/gdb)
to debug programs. Currently, programs are assumed to be written in C.

Internally, this package relies
on the [GDB/MI](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI.html)
API.

## Using Aili-GDBState

Start by implementing the `GdbMiSession` trait, either directly, or by implementing
the simpler `GdbMiStream` or `GdbMiStringStream`. These should provide access
to the GDB API.

Next, construct the `GdbStateGraph`, which can be used with the rest of Aili.

## Documentation

The following command generates documentation and saves it
in the target directory.

```sh
cargo doc --no-deps
```

## Tests

```sh
cargo test
```

Besides unit test suites, this package contains integration tests
that verify the implementation against a real instance of GDB running
a compiled program. This means GDB and a C compiler need to be present
to run the test. By default, the tests assume these programs are included
in `PATH`, but this may be modified by environment variables.

```sh
GDB_PATH=/bin/gdb CC_PATH=/bin/gcc cargo test --test integration_test
```
