//! Simple functions to add, monitor and configure network interfaces.

use crate::{Error, Result};

use std::time::Duration;

use tokio::time::sleep;

use futures::TryStreamExt;
use netlink_packet_route::rtnl::IFF_UP;

/// Brings an interface up or down.
#[cfg(feature = "link")]
pub async fn set(link: String, state: bool) -> Result<()> {
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

    match state {
        true => handle.link().set(id).up(),
        false => handle.link().set(id).down(),
    }
    .execute()
    .await?;

    Ok(())
}

/// Reports whether an interface is up.
///
/// # Errors
///
/// This function fails if the interface doesn't exist
/// or if any other `rtnetlink` error occurs.
pub async fn is_up(link: String) -> Result<bool> {
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

    let is_up = link.header.flags & IFF_UP == IFF_UP;
    Ok(is_up)
}

/// Sets the MTU of an interface.
#[cfg(feature = "link")]
pub async fn set_mtu(link: String, mtu: u32) -> Result<()> {
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

    handle.link().set(id).mtu(mtu).execute().await?;
    Ok(())
}

/// Creates a VLAN interface on top of a parent interface.
///
/// # Arguments
///
/// * `link` - The name of the VLAN interface to be created.
/// * `parent` - The name of the parent interface for the actual traffic.
/// * `vlan_id` - The VLAN ID for tagging.
#[cfg(feature = "link")]
pub async fn add_vlan(link: String, parent: String, vlan_id: u16) -> Result<()> {
    let (conn, handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(conn);

    let parent = handle
        .link()
        .get()
        .match_name(parent.clone())
        .execute()
        .try_next()
        .await?
        .ok_or(Error::LinkNotFound(parent))?;

    let parent_id = parent.header.index;

    handle
        .link()
        .add()
        .vlan(link, parent_id, vlan_id)
        .execute()
        .await?;

    Ok(())
}

/// Waits for an interface to come up, including waiting for its creation.
pub async fn wait_up(link: String) -> Result<()> {
    while !exists(link.clone()).await? || !is_up(link.clone()).await? {
        sleep(Duration::from_millis(200)).await;
    }

    Ok(())
}

/// Reports whether an interface exists.
pub async fn exists(link: String) -> Result<bool> {
    let (conn, handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(conn);

    let exists = handle
        .link()
        .get()
        .match_name(link)
        .execute()
        .try_next()
        .await
        .is_ok();

    Ok(exists)
}

/// Waits until an interface is created.
pub async fn wait_exists(link: String) -> Result<()> {
    while !exists(link.clone()).await? {
        sleep(Duration::from_millis(200)).await;
    }

    Ok(())
}

/// Returns the index of an interface.
pub async fn index(link: String) -> Result<u32> {
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

    Ok(link.header.index)
}
