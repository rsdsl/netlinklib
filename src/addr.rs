//! Simple functions to add and delete IP addresses.

use crate::{Error, Result};

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use futures::{future, TryStream, TryStreamExt};
use netlink_packet_route::{
    address::Nla, AddressMessage, AF_INET, AF_INET6, RT_SCOPE_LINK, RT_SCOPE_UNIVERSE,
};

/// Flushes all addresses of an interface.
pub async fn flush(link: String) -> Result<()> {
    let (conn, handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(conn);

    let link = handle
        .link()
        .get()
        .match_name(link.clone())
        .execute()
        .try_next()
        .await?
        .ok_or(Error::LinkNotFound(link))?;

    let id = link.header.index;

    let addrs: Vec<AddressMessage> = handle
        .address()
        .get()
        .set_link_index_filter(id)
        .execute()
        .try_collect()
        .await?;

    for addr in addrs {
        handle.address().del(addr).execute().await?;
    }

    Ok(())
}

/// Flushes the IPv4 addresses of an interface.
pub async fn flush4(link: String) -> Result<()> {
    let (conn, handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(conn);

    let link = handle
        .link()
        .get()
        .match_name(link.clone())
        .execute()
        .try_next()
        .await?
        .ok_or(Error::LinkNotFound(link))?;

    let id = link.header.index;

    let addrs: Vec<AddressMessage> = handle
        .address()
        .get()
        .set_link_index_filter(id)
        .execute()
        .try_filter(|addr| future::ready(addr.header.family == AF_INET as u8))
        .try_collect()
        .await?;

    for addr in addrs {
        handle.address().del(addr).execute().await?;
    }

    Ok(())
}

/// Flushes the IPv6 addresses of an interface.
pub async fn flush6(link: String) -> Result<()> {
    let (conn, handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(conn);

    let link = handle
        .link()
        .get()
        .match_name(link.clone())
        .execute()
        .try_next()
        .await?
        .ok_or(Error::LinkNotFound(link))?;

    let id = link.header.index;

    let addrs: Vec<AddressMessage> = handle
        .address()
        .get()
        .set_link_index_filter(id)
        .execute()
        .try_filter(|addr| future::ready(addr.header.family == AF_INET6 as u8))
        .try_collect()
        .await?;

    for addr in addrs {
        handle.address().del(addr).execute().await?;
    }

    Ok(())
}

/// Flushes all global unicast IPv6 addresses from all interfaces.
pub async fn flush6_global() -> Result<()> {
    let (conn, handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(conn);

    let addrs: Vec<AddressMessage> = handle
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
        handle.address().del(addr).execute().await?;
    }

    Ok(())
}

/// Adds an IP address to an interface.
pub async fn add(link: String, addr: IpAddr, prefix_len: u8) -> Result<()> {
    let (conn, handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(conn);

    let link = handle
        .link()
        .get()
        .match_name(link.clone())
        .execute()
        .try_next()
        .await?
        .ok_or(Error::LinkNotFound(link))?;

    let id = link.header.index;

    handle.address().add(id, addr, prefix_len).execute().await?;

    Ok(())
}

/// Adds a link-scope IP address to an interface.
/// This is especially useful with IPv6.
pub async fn add_link_local(link: String, addr: IpAddr, prefix_len: u8) -> Result<()> {
    let (conn, handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(conn);

    let link = handle
        .link()
        .get()
        .match_name(link.clone())
        .execute()
        .try_next()
        .await?
        .ok_or(Error::LinkNotFound(link))?;

    let id = link.header.index;

    let mut req = handle.address().add(id, addr, prefix_len);
    req.message_mut().header.scope = RT_SCOPE_LINK;

    req.execute().await?;

    Ok(())
}

/// Returns an iterator over the IP addresses of an interface.
pub async fn get(link: String) -> Result<impl TryStream<Ok = IpAddr, Error = Error>> {
    let (conn, handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(conn);

    let link = handle
        .link()
        .get()
        .match_name(link.clone())
        .execute()
        .try_next()
        .await?
        .ok_or(Error::LinkNotFound(link))?;

    Ok(handle
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
