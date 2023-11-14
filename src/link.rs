use crate::{Error, Result};

use std::time::Duration;

use tokio::time::sleep;

use futures::TryStreamExt;
use netlink_packet_route::rtnl::IFF_UP;

#[cfg(feature = "link")]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LinkState {
    Up,
    Down,
}

#[cfg(feature = "link")]
pub async fn set(link: String, state: LinkState) -> Result<()> {
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
        LinkState::Up => handle.link().set(id).up(),
        LinkState::Down => handle.link().set(id).down(),
    }
    .execute()
    .await?;

    Ok(())
}

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

pub async fn wait_up(link: String) -> Result<()> {
    while !exists(link.clone()).await? || !is_up(link.clone()).await? {
        sleep(Duration::from_millis(200)).await;
    }

    Ok(())
}

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

pub async fn wait_exists(link: String) -> Result<()> {
    while !exists(link.clone()).await? {
        sleep(Duration::from_millis(200)).await;
    }

    Ok(())
}
