// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

// mod checksum;
pub mod datagram;
pub(crate) mod endpoint;
mod peer;

#[cfg(test)]
mod tests;

pub use datagram::{Ipv4Header, Ipv4Protocol2};
pub use endpoint::Ipv4Endpoint as Endpoint;
pub use peer::Ipv4Peer as Peer;
