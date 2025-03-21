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
pub struct Connection {
    rt: tokio::runtime::Runtime,
    conn: crate::Connection,
}

impl Connection {
    /// Creates a new blocking wrapper around [`crate::Connection`].
    pub fn new() -> crate::Result<Self> {
        let rt = tokio::runtime::Runtime::new()?;

        Ok(Self {
            conn: rt.block_on(crate::Connection::new())?,
            rt,
        })
    }
}

macro_rules! blockify {
    ($blk:ident) => {
        pub fn $blk(&self) -> crate::Result<()> {
            self.rt.block_on(self.conn.$blk())
        }
    };
    ($blk:ident, $($v:tt: $t:ty),*) => {
        pub fn $blk(&self, $($v: $t),*) -> crate::Result<()> {
            self.rt.block_on(self.conn.$blk($($v),*))
        }
    };
    ($blk:ident -> $ret:ty, $($v:tt: $t:ty),*) => {
        pub fn $blk(&self, $($v: $t),*) -> crate::Result<$ret> {
            self.rt.block_on(self.conn.$blk($($v),*))
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

        pub fn address_get(&self, link: String) -> crate::Result<Vec<IpAddr>> {
            self.rt
                .block_on(async { self.conn.address_get(link).await?.try_collect().await })
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
        #[cfg(feature = "link")]
        blockify!(link_add_wireguard, link: String);
        #[cfg(feature = "link")]
        blockify!(link_delete, link: String);

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

    use crate::route::{Route4, Route6};

    impl Connection {
        blockify!(route_flush4, link: String);
        blockify!(route_flush6, link: String);
        blockify!(route_flush, link: String);
        blockify!(route_add4, r: Route4);
        blockify!(route_add6, r: Route6);
        blockify!(route_del4, r: Route4);
        blockify!(route_del6, r: Route6);
    }
}

#[cfg(feature = "rule")]
pub mod rule {
    use super::Connection;

    use std::net::{Ipv4Addr, Ipv6Addr};

    use crate::rule::Rule;
    use crate::Result;

    impl Rule<()> {
        pub fn blocking_add(self, c: &Connection) -> Result<()> {
            c.rt.block_on(self.add(&c.conn))
        }

        pub fn blocking_del(self, c: &Connection) -> Result<()> {
            c.rt.block_on(self.del(&c.conn))
        }
    }

    impl Rule<Ipv4Addr> {
        pub fn blocking_add(self, c: &Connection) -> Result<()> {
            c.rt.block_on(self.add(&c.conn))
        }

        pub fn blocking_del(self, c: &Connection) -> Result<()> {
            c.rt.block_on(self.del(&c.conn))
        }
    }

    impl Rule<Ipv6Addr> {
        pub fn blocking_add(self, c: &Connection) -> Result<()> {
            c.rt.block_on(self.add(&c.conn))
        }

        pub fn blocking_del(self, c: &Connection) -> Result<()> {
            c.rt.block_on(self.del(&c.conn))
        }
    }
}
