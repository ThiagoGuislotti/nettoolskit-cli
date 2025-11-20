//! Templating Test Suite Entry Point
//!
//! Main test suite aggregator for nettoolskit-templating crate.
//! Test structure mirrors the src/ directory structure:
//! - core/: Engine and error handling tests
//! - rendering/: Batch renderer and resolver tests (includes common test utilities)
//! - strategies/: Language-specific strategy and factory tests

// Test modules mirroring src/ structure
mod core;
mod rendering;
mod strategies;