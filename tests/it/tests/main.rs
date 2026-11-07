// SPDX-License-Identifier: AGPL-3.0-or-later
//! The integration tests (plan section 16), as one test binary: each file here is a module. One binary links once, and
//! macOS scans a newly built executable once on its first launch (about 10 s on this Mac, 2026-11-03), where 36
//! binaries paid that 36 times. A new test file goes in the list below.
mod agent_claude;
mod agent_codex;
mod agent_sentinel;
mod bom;
mod bus;
mod cancel;
mod chatgpt;
mod check;
mod cli;
mod crates;
mod dsl;
mod errors;
mod eval;
mod export;
mod frame;
mod guides;
mod host;
mod import;
mod instructions;
mod ladder;
mod ldraw;
mod library;
mod mcp;
mod parts_data;
mod paths;
mod python;
mod python_setup;
mod refs;
mod render;
mod run;
mod runner;
mod search;
mod setup;
mod technic;
mod techniques;
mod validator_bench;
mod validator_corpus;
mod validator_mutations;
mod versions;
