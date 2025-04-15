// #![allow(unused)]
// message
#[cfg(feature = "message")]
pub mod message;

// traits
#[cfg(feature = "traits")]
pub mod traits;

// router
#[cfg(feature = "router")]
pub use router::Router;
#[cfg(feature = "router")]
mod router;

// error
#[cfg(feature = "error")]
pub mod error;
#[cfg(feature = "error")]
pub use error::Error;

// topic
#[cfg(feature = "topic")]
mod topics;
#[cfg(feature = "topic")]
pub use {macros::topic_macro as topic, topics::Topic};

// default-pubsub
#[cfg(feature = "default-pubsub")]
mod pubsub;
#[cfg(feature = "default-pubsub")]
pub use pubsub::PubSub;

// bus
#[cfg(feature = "bus")]
mod bus;
#[cfg(feature = "bus")]
pub use bus::Bus;
