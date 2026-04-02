#![allow(non_snake_case)]

/// WFP Callout Library
/// ===================
/// Windows Filtering Platform kernel-mode callout for packet interception and rate limiting

mod engine;
mod pipe;

pub use engine::WfpEngine;

// Re-export proto types for use by this crate
extern crate proto;

// Note: This crate runs in kernel mode with limited functionality
// All business logic happens in the daemon userspace process
