//! Fuzz target for IPC JSON message decoding.
//!
//! Tests that arbitrary byte sequences cannot cause panics or memory issues
//! when parsed as IPC messages.

#![no_main]

use libfuzzer_sys::fuzz_target;
use gg_core::ipc::decode_message;

fuzz_target!(|data: &[u8]| {
    // Attempt to decode arbitrary bytes as an IPC message.
    // This should never panic - only return Ok or Err.
    let _ = decode_message(data);
});
