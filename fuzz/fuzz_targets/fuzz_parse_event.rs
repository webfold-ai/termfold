//! Fuzz the terminal input parser: arbitrary bytes in, `Event` out.
//!
//! This is the only untrusted-byte entry point in the compiled crate. A panic, overflow or
//! unbounded loop here is reachable from anything that writes to the terminal's input stream.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = termfold::terminal::parse_event(data);
});
