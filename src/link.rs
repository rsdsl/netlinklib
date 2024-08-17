//! Simple functions to add, monitor and configure network interfaces.

use crate::{Connection, Error, Result};

use std::time::Duration;

use tokio::time::sleep;

use futures::TryStreamExt;
use netlink_packet_route::link::LinkFlag;

impl Connection {
    /// Brings an interface up or down.
    #[cfg(feature = "link")]
    pub async fn link_set(&self, link: String, state: bool) -> Result<()> {
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

        match state {
            true => self.handle().link().set(id).up(),
            false => self.handle().link().set(id).down(),
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
    pub async fn link_is_up(&self, link: String) -> Result<bool> {
        let link = self
            .handle()
            .link()
            .get()
            .match_name(link.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(link))?;

        let is_up = link.header.flags.iter().any(|flag| *flag == LinkFlag::Up);
        Ok(is_up)
    }

    /// Sets the MTU of an interface.
    #[cfg(feature = "link")]
    pub async fn link_set_mtu(&self, link: String, mtu: u32) -> Result<()> {
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

        self.handle().link().set(id).mtu(mtu).execute().await?;
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
    pub async fn link_add_vlan(&self, link: String, parent: String, vlan_id: u16) -> Result<()> {
        let parent = self
            .handle()
            .link()
            .get()
            .match_name(parent.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(parent))?;

        let parent_id = parent.header.index;

        self.handle()
            .link()
            .add()
            .vlan(link, parent_id, vlan_id)
            .execute()
            .await?;

        Ok(())
    }

    /// Waits for an interface to come up, including waiting for its creation.
    pub async fn link_wait_up(&self, link: String) -> Result<()> {
        while !self.link_exists(link.clone()).await? || !self.link_is_up(link.clone()).await? {
            sleep(Duration::from_millis(200)).await;
        }

        Ok(())
    }

    /// Reports whether an interface exists.
    pub async fn link_exists(&self, link: String) -> Result<bool> {
        let exists = self
            .handle()
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
    pub async fn link_wait_exists(&self, link: String) -> Result<()> {
        while !self.link_exists(link.clone()).await? {
            sleep(Duration::from_millis(200)).await;
        }

        Ok(())
    }

    /// Returns the index of an interface.
    pub async fn link_index(&self, link: String) -> Result<u32> {
        let link = self
            .handle()
            .link()
            .get()
            .match_name(link.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(link))?;

        Ok(link.header.index)
    }
}
