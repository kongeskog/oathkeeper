//! Shared data and arithmetic. No evaluator, rule selection, filesystem, or network.
mod model;
mod validation;
mod value;
mod wire;
pub use model::*;
pub use validation::*;
pub use value::*;
pub use wire::*;
