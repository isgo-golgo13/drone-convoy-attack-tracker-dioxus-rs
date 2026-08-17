//! # Services Module
//!
//! GraphQL HTTP client (queries + mutations). The dashboard runs on a 2 s
//! poll; `/graphql/ws` subscriptions are the Tier 3 upgrade path (see
//! context-handoff PART 4) and will land here as a sibling module.

pub mod api;

pub use api::*;
