// allow match over single value, as it is used frequently for matching on photon messages
#![allow(clippy::single_match)]

pub mod hax;
pub(crate) mod proxy;
pub mod version_scraper;

pub use photon_lib::indexmap;
pub use tokio_tungstenite::tungstenite;
