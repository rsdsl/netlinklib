//! Simple functions to add and delete routes.

use crate::{Error, Result};

use std::net::{Ipv4Addr, Ipv6Addr};

use futures::{future, TryStreamExt};
use netlink_packet_route::{RouteMessage, RT_SCOPE_LINK};
use rtnetlink::IpVersion;

/// Flushes all IPv4 routes from an interface.
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

    let routes: Vec<RouteMessage> = handle
        .route()
        .get(IpVersion::V4)
        .execute()
        .try_filter(|route| {
            future::ready(if let Some(ifi) = route.output_interface() {
                ifi == id
            } else {
                false
            })
        })
        .try_collect()
        .await?;

    for route in routes {
        handle.route().del(route).execute().await?;
    }

    Ok(())
}

/// Flushes all IPv6 routes from an interface.
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

    let routes: Vec<RouteMessage> = handle
        .route()
        .get(IpVersion::V6)
        .execute()
        .try_filter(|route| {
            future::ready(if let Some(ifi) = route.output_interface() {
                ifi == id
            } else {
                false
            })
        })
        .try_collect()
        .await?;

    for route in routes {
        handle.route().del(route).execute().await?;
    }

    Ok(())
}

/// Adds a simple IPv4 route with an optional gateway.
pub async fn add4(
    dst: Ipv4Addr,
    prefix_len: u8,
    rtr: Option<Ipv4Addr>,
    link: String,
) -> Result<()> {
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

    let mut add = handle
        .route()
        .add()
        .v4()
        .destination_prefix(dst, prefix_len)
        .output_interface(id);

    if let Some(rtr) = rtr {
        add = add.gateway(rtr);
    } else {
        add = add.scope(RT_SCOPE_LINK);
    }

    add.execute().await?;
    Ok(())
}

/// Adds a simple IPv6 route with an optional gateway.
pub async fn add6(
    dst: Ipv6Addr,
    prefix_len: u8,
    rtr: Option<Ipv6Addr>,
    link: String,
) -> Result<()> {
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

    let mut add = handle
        .route()
        .add()
        .v6()
        .destination_prefix(dst, prefix_len)
        .output_interface(id);

    if let Some(rtr) = rtr {
        add = add.gateway(rtr);
    } else {
        add = add.scope(RT_SCOPE_LINK);
    }

    add.execute().await?;
    Ok(())
}
