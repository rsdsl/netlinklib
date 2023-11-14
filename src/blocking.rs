//! Blocking wrappers around the asynchronous API.
//!
//! All functions except for members of the tunnel module internally use their
//! async counterparts inside a temporary tokio runtime.
//! Tunnels are already synchronous.
//!
//! Consult the async modules for documentation.

#[cfg(feature = "tunnel")]
pub use crate::tunnel;

macro_rules! blockify {
    ($blk:ident, $r:path) => {
        pub fn $blk() -> crate::Result<()> {
            tokio::runtime::Runtime::new()?.block_on($r())
        }
    };
    ($blk:ident, $r:path, $($v:tt: $t:ty),*) => {
        pub fn $blk($($v: $t),*) -> crate::Result<()> {
            tokio::runtime::Runtime::new()?.block_on($r($($v),*))
        }
    };
    ($blk:ident -> $ret:ty, $r:path, $($v:tt: $t:ty),*) => {
        pub fn $blk($($v: $t),*) -> crate::Result<$ret> {
            tokio::runtime::Runtime::new()?.block_on($r($($v),*))
        }
    };
}

#[cfg(feature = "addr")]
pub mod addr {
    use crate::addr;

    use std::net::IpAddr;

    blockify!(flush, addr::flush, link: String);
    blockify!(flush4, addr::flush4, link: String);
    blockify!(flush6, addr::flush6, link: String);
    blockify!(flush6_global, addr::flush6_global);
    blockify!(add, addr::add, link: String, addr: IpAddr, prefix_len: u8);
    blockify!(add_link_local, addr::add_link_local, link: String, addr: IpAddr, prefix_len: u8);
}

#[cfg(feature = "status")]
pub mod link {
    use crate::link::{self, LinkState};

    #[cfg(feature = "link")]
    blockify!(set, link::set, link: String, state: LinkState);
    #[cfg(feature = "link")]
    blockify!(set_mtu, link::set_mtu, link: String, mtu: u32);
    #[cfg(feature = "link")]
    blockify!(add_vlan, link::add_vlan, link: String, parent: String, vlan_id: u16);

    blockify!(is_up -> bool, link::is_up, link: String);
    blockify!(wait_up, link::wait_up, link: String);
    blockify!(exists -> bool, link::exists, link: String);
    blockify!(wait_exists, link::wait_exists, link: String);
}

#[cfg(feature = "route")]
pub mod route {
    use crate::route;

    use std::net::{Ipv4Addr, Ipv6Addr};

    blockify!(flush4, route::flush4, link: String);
    blockify!(flush6, route::flush6, link: String);
    blockify!(add4, route::add4, dst: Ipv4Addr, prefix_len: u8, rtr: Option<Ipv4Addr>, link: String);
    blockify!(add6, route::add6, dst: Ipv6Addr, prefix_len: u8, rtr: Option<Ipv6Addr>, link: String);
}
