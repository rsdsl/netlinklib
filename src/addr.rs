//! Simple functions to add and delete IP addresses.

use crate::{Connection, Error, Result};

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use futures::{future, TryStream, TryStreamExt};
use netlink_packet_route::{
    address::Nla, AddressMessage, AF_INET, AF_INET6, RT_SCOPE_LINK, RT_SCOPE_UNIVERSE,
};

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
            .try_filter(|addr| future::ready(addr.header.family == AF_INET as u8))
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
            .try_filter(|addr| future::ready(addr.header.family == AF_INET6 as u8))
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
                    addr.header.family == AF_INET6 as u8 && addr.header.scope == RT_SCOPE_UNIVERSE,
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
        req.message_mut().header.scope = RT_SCOPE_LINK;

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
                future::ready(Ok(if let Some(Nla::Address(bytes)) = msg.nlas.first() {
                    match msg.header.family as u16 {
                        AF_INET => {
                            let octets: [u8; 4] = (*bytes)
                                .clone()
                                .try_into()
                                .expect("nla does not match ipv4 address length");
                            let ip = IpAddr::from(Ipv4Addr::from(octets));

                            Some(ip)
                        }
                        AF_INET6 => {
                            let octets: [u8; 16] = (*bytes)
                                .clone()
                                .try_into()
                                .expect("nla does not match ipv6 address length");
                            let ip = IpAddr::from(Ipv6Addr::from(octets));

                            Some(ip)
                        }
                        _ => None,
                    }
                } else {
                    None
                }))
            }))
    }
}
