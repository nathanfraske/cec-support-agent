// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Corpus client, schema, and cold-start store.
//!
//! This crate is the only contact point with the private corpus service. It
//! ships **no corpus data** (invariant 2): at cold start it runs against an
//! empty in-memory [`LocalCorpus`] and a local inference endpoint, with no
//! CEC-hosted service required (invariant 3).
//!
//! # Sign-off gate (invariant 6)
//! [`CorpusStore::submit`] refuses any [`Contribution`] whose [`SignOff`] is not
//! `VerifierConfirmed` or `HumanConfirmed`. The refusal is enforced here in
//! code via [`ensure_signed_off`] — not in documentation — and every submit
//! path (local and remote) calls it before persisting or transmitting anything.

mod gate;
mod schema;
mod store;

pub use gate::{ensure_signed_off, GateError};
pub use schema::{Contribution, FixMapping, Outcome, SignOff};
pub use store::{CorpusError, CorpusStore, HttpCorpus, LocalCorpus};
