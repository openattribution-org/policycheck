//! Compliance check modules.
//!
//! Each module implements analysis for a specific web compliance standard.
//! Modules that parse robots.txt receive the raw content as `&str`.
//! Modules with their own data source (e.g. TDM) receive pre-fetched data.
//!
//! To add a new standard, create a new module here and wire it into
//! `PolicyAnalyzer::analyze()` in `lib.rs`.

pub mod ai_bots;
pub mod content_signals;
pub mod markdown_agents;
pub mod robots;
pub mod robots_meta;
pub mod rsl;
pub mod tdm;
pub mod well_known_oa;
