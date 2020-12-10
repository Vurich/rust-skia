#![allow(clippy::all)]
// GrVkBackendContext contains u128 fields on macOS
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
