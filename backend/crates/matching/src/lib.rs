pub mod config;
pub mod engine;
pub mod scorers;
pub mod trace;

pub use config::MatchingConfig;
pub use engine::{evaluate, MatchResult};
pub use trace::RuleTrace;
