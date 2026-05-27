//! Synergism Forkd — test fixtures, mock state builders, seeded RNG,
//! headless sim runner, and parity-snapshot helpers.
//!
//! Imported by every other crate's tests. The companion
//! `synergismforkd-sim` binary drives the same sim runner from the CLI
//! for ad-hoc balance checks and bug reproduction.

use synergismforkd_bignum as _;
use synergismforkd_logic as _;
use synergismforkd_save as _;

pub fn placeholder() {}
