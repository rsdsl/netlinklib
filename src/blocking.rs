//! Blocking wrappers around the asynchronous API.
//!
//! All functions except for members of the tunnel module internally use their
//! async counterparts inside a temporary tokio runtime.
//! Tunnels are already synchronous.
//!
//! Consult the async modules for documentation.

#[cfg(feature = "tunnel")]
pub use crate::tunnel;

/// A blocking wrapper around the async [`crate::Connection`].
#[derive(Debug)]
pub struct Connection(crate::Connection);

macro_rules! blockify {
    ($blk:ident) => {
        pub fn $blk(&self) -> crate::Result<()> {
            tokio::runtime::Runtime::new()?.block_on(self.0.$blk())
        }
    };
    ($blk:ident, $($v:tt: $t:ty),*) => {
        pub fn $blk(&self, $($v: $t),*) -> crate::Result<()> {
            tokio::runtime::Runtime::new()?.block_on(self.0.$blk($($v),*))
        }
    };
    ($blk:ident -> $ret:ty, $($v:tt: $t:ty),*) => {
        pub fn $blk(&self, $($v: $t),*) -> crate::Result<$ret> {
            tokio::runtime::Runtime::new()?.block_on(self.0.$blk($($v),*))
        }
    };
}

#[cfg(feature = "addr")]
pub mod addr {
    use super::Connection;

    use std::net::IpAddr;

    use futures::TryStreamExt;

    impl Connection {
        blockify!(address_flush, link: String);
        blockify!(address_flush4, link: String);
        blockify!(address_flush6, link: String);
        blockify!(address_flush6_global);
        blockify!(address_add, link: String, addr: IpAddr, prefix_len: u8);
        blockify!(address_add_link_local, link: String, addr: IpAddr, prefix_len: u8);

        pub fn get(&self, link: String) -> crate::Result<Vec<IpAddr>> {
            tokio::runtime::Runtime::new()?
                .block_on(async { self.0.address_get(link).await?.try_collect().await })
        }
    }
}

#[cfg(feature = "status")]
pub mod link {
    use super::Connection;

    impl Connection {
        #[cfg(feature = "link")]
        blockify!(link_set, link: String, state: bool);
        #[cfg(feature = "link")]
        blockify!(link_set_mtu, link: String, mtu: u32);
        #[cfg(feature = "link")]
        blockify!(link_add_vlan, link: String, parent: String, vlan_id: u16);

        blockify!(link_is_up -> bool, link: String);
        blockify!(link_wait_up, link: String);
        blockify!(link_exists -> bool, link: String);
        blockify!(link_wait_exists, link: String);
        blockify!(link_index -> u32, link: String);
    }
}

#[cfg(feature = "route")]
pub mod route {
    use super::Connection;

    use std::net::{Ipv4Addr, Ipv6Addr};

    impl Connection {
        blockify!(route_flush4, link: String);
        blockify!(route_flush6, link: String);
        blockify!(route_flush, link: String);
        blockify!(route_add4, dst: Ipv4Addr, prefix_len: u8, rtr: Option<Ipv4Addr>, link: String);
        blockify!(route_add6, dst: Ipv6Addr, prefix_len: u8, rtr: Option<Ipv6Addr>, link: String);
    }
}
