pub mod display_name;
pub mod email;
pub mod service;
pub mod team;
pub mod username;

use ovia_db::identity::models::{Identity, Person};

use crate::trace::ScorerResult;

pub trait Scorer {
    fn name(&self) -> &'static str;
    fn score(&self, person: &Person, identity: &Identity) -> ScorerResult;
}
