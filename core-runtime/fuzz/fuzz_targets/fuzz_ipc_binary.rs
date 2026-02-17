//! Fuzz target for IPC binary (bincode) message decoding.
//!
//! Tests that arbitrary byte sequences cannot cause panics or memory issues
//! when parsed as binary IPC messages.

#![no_main]

use libfuzzer_sys::fuzz_target;
use veritas_sdr::ipc::decode_message_binary;

fuzz_target!(|data: &[u8]| {
    // Attempt to decode arbitrary bytes as a binary IPC message.
    // This should never panic - only return Ok or Err.
    let _ = decode_message_binary(data);
});
