//! Simple functions to add and delete IP addresses.

use crate::{Connection, Error, Result};

use std::net::IpAddr;

use futures::{future, TryStream, TryStreamExt};
use netlink_packet_route::address::{AddressAttribute, AddressMessage, AddressScope};
use netlink_packet_route::AddressFamily;

impl Connection {
    /// Flushes all addresses of an interface.
    pub async fn address_flush(&self, link: String) -> Result<()> {
        let link = self
            .handle()
            .link()
            .get()
            .match_name(link.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(link))?;

        let id = link.header.index;

        let addrs: Vec<AddressMessage> = self
            .handle()
            .address()
            .get()
            .set_link_index_filter(id)
            .execute()
            .try_collect()
            .await?;

        for addr in addrs {
            self.handle().address().del(addr).execute().await?;
        }

        Ok(())
    }

    /// Flushes the IPv4 addresses of an interface.
    pub async fn address_flush4(&self, link: String) -> Result<()> {
        let link = self
            .handle()
            .link()
            .get()
            .match_name(link.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(link))?;

        let id = link.header.index;

        let addrs: Vec<AddressMessage> = self
            .handle()
            .address()
            .get()
            .set_link_index_filter(id)
            .execute()
            .try_filter(|addr| future::ready(addr.header.family == AddressFamily::Inet))
            .try_collect()
            .await?;

        for addr in addrs {
            self.handle().address().del(addr).execute().await?;
        }

        Ok(())
    }

    /// Flushes the IPv6 addresses of an interface.
    pub async fn address_flush6(&self, link: String) -> Result<()> {
        let link = self
            .handle()
            .link()
            .get()
            .match_name(link.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(link))?;

        let id = link.header.index;

        let addrs: Vec<AddressMessage> = self
            .handle()
            .address()
            .get()
            .set_link_index_filter(id)
            .execute()
            .try_filter(|addr| future::ready(addr.header.family == AddressFamily::Inet6))
            .try_collect()
            .await?;

        for addr in addrs {
            self.handle().address().del(addr).execute().await?;
        }

        Ok(())
    }

    /// Flushes all global unicast IPv6 addresses from all interfaces.
    pub async fn address_flush6_global(&self) -> Result<()> {
        let addrs: Vec<AddressMessage> = self
            .handle()
            .address()
            .get()
            .execute()
            .try_filter(|addr| {
                future::ready(
                    addr.header.family == AddressFamily::Inet6
                        && addr.header.scope == AddressScope::Universe,
                )
            })
            .try_collect()
            .await?;

        for addr in addrs {
            self.handle().address().del(addr).execute().await?;
        }

        Ok(())
    }

    /// Adds an IP address to an interface.
    pub async fn address_add(&self, link: String, addr: IpAddr, prefix_len: u8) -> Result<()> {
        let link = self
            .handle()
            .link()
            .get()
            .match_name(link.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(link))?;

        let id = link.header.index;

        self.handle()
            .address()
            .add(id, addr, prefix_len)
            .execute()
            .await?;

        Ok(())
    }

    /// Adds a link-scope IP address to an interface.
    /// This is especially useful with IPv6.
    pub async fn address_add_link_local(
        &self,
        link: String,
        addr: IpAddr,
        prefix_len: u8,
    ) -> Result<()> {
        let link = self
            .handle()
            .link()
            .get()
            .match_name(link.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(link))?;

        let id = link.header.index;

        let mut req = self.handle().address().add(id, addr, prefix_len);
        req.message_mut().header.scope = AddressScope::Link;

        req.execute().await?;

        Ok(())
    }

    /// Returns an iterator over the IP addresses of an interface.
    pub async fn address_get(
        &self,
        link: String,
    ) -> Result<impl TryStream<Ok = IpAddr, Error = Error>> {
        let link = self
            .handle()
            .link()
            .get()
            .match_name(link.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(link))?;

        Ok(self
            .handle()
            .address()
            .get()
            .set_link_index_filter(link.header.index)
            .execute()
            .err_into::<Error>()
            .try_filter_map(|msg| {
                future::ready(Ok(
                    if let Some(AddressAttribute::Address(ip)) = msg.attributes.first() {
                        match msg.header.family {
                            AddressFamily::Inet | AddressFamily::Inet6 => Some(*ip),
                            _ => None,
                        }
                    } else {
                        None
                    },
                ))
            }))
    }
}
