//! Repository module.
//!
//! Splits responsibilities into Tasks (structure) and Proofs (verification).

pub mod proofs;
pub mod tasks;

pub use proofs::ProofRepo;
pub use tasks::{TaskRepo, TASK_SELECT};