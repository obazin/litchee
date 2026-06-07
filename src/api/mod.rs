//! The Lichess API, organized by business concern.
//!
//! Each submodule groups related concerns into a category; every concern
//! exposes its endpoints through an accessor on
//! [`LichessClient`](crate::LichessClient) (for example `client.account()` or
//! `client.broadcasts()`).

pub mod auth;
pub mod broadcasting;
pub mod database;
pub mod engine;
pub mod gameplay;
pub mod social;
pub mod tournaments;
pub mod training;
pub mod users;
