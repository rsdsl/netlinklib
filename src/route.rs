//! Simple functions to add and delete routes.

use crate::{Connection, Error, Result};

use std::net::{Ipv4Addr, Ipv6Addr};

use futures::{future, TryStreamExt};
use netlink_packet_route::route::{RouteAttribute, RouteMessage, RouteScope};
use rtnetlink::IpVersion;

/// An IPv4 route configuration.
#[derive(Clone, Debug)]
pub struct Route4 {
    /// The destination prefix this route applies to.
    pub dst: Ipv4Addr,
    /// The length of the destination prefix this route applies to.
    pub prefix_len: u8,
    /// The (optional) router to send packets to.
    pub rtr: Option<Ipv4Addr>,
    /// Whether to apply the link scope to this route.
    pub on_link: bool,
    /// The table this route belongs to. Defaults to `main`.
    pub table: Option<u32>,
    /// The metric (priority) of this route. `None` causes the kernel default
    /// to be used.
    pub metric: Option<u32>,
    /// The network interface to send packets over.
    pub link: String,
}

/// An IPv6 route configuration.
#[derive(Clone, Debug)]
pub struct Route6 {
    /// The destination prefix this route applies to.
    pub dst: Ipv6Addr,
    /// The length of the destination prefix this route applies to.
    pub prefix_len: u8,
    /// The (optional) router to send packets to.
    pub rtr: Option<Ipv6Addr>,
    /// Whether to apply the link scope to this route.
    pub on_link: bool,
    /// The table this route belongs to. Defaults to `main`.
    pub table: Option<u32>,
    /// The metric (priority) of this route. `None` causes the kernel default
    /// to be used.
    pub metric: Option<u32>,
    /// The network interface to send packets over.
    pub link: String,
}

impl Connection {
    /// Flushes all IPv4 routes from an interface.
    pub async fn route_flush4(&self, link: String) -> Result<()> {
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

        let routes: Vec<RouteMessage> = self
            .handle()
            .route()
            .get(IpVersion::V4)
            .execute()
            .try_filter(|route| {
                future::ready(
                    if let Some(ifi) = route.attributes.iter().find_map(|attr| {
                        if let RouteAttribute::Oif(oif) = *attr {
                            Some(oif)
                        } else {
                            None
                        }
                    }) {
                        ifi == id
                    } else {
                        false
                    },
                )
            })
            .try_collect()
            .await?;

        for route in routes {
            self.handle().route().del(route).execute().await?;
        }

        Ok(())
    }

    /// Flushes all IPv6 routes from an interface.
    pub async fn route_flush6(&self, link: String) -> Result<()> {
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

        let routes: Vec<RouteMessage> = self
            .handle()
            .route()
            .get(IpVersion::V6)
            .execute()
            .try_filter(|route| {
                future::ready(
                    if let Some(ifi) = route.attributes.iter().find_map(|attr| {
                        if let RouteAttribute::Oif(oif) = *attr {
                            Some(oif)
                        } else {
                            None
                        }
                    }) {
                        ifi == id
                    } else {
                        false
                    },
                )
            })
            .try_collect()
            .await?;

        for route in routes {
            self.handle().route().del(route).execute().await?;
        }

        Ok(())
    }

    /// Flushes all routes from an interface.
    pub async fn route_flush(&self, link: String) -> Result<()> {
        self.route_flush4(link.clone()).await?;
        self.route_flush6(link).await?;

        Ok(())
    }

    /// Adds a simple IPv4 route with an optional gateway.
    pub async fn route_add4(&self, r: Route4) -> Result<()> {
        let link = self
            .handle()
            .link()
            .get()
            .match_name(r.link.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(r.link))?;

        let id = link.header.index;

        let mut add = self
            .handle()
            .route()
            .add()
            .v4()
            .destination_prefix(r.dst, r.prefix_len)
            .output_interface(id);

        if let Some(rtr) = r.rtr {
            add = add.gateway(rtr);
        }

        if r.on_link {
            add = add.scope(RouteScope::Link);
        }

        if let Some(table) = r.table {
            add = add.table_id(table);
        }

        if let Some(metric) = r.metric {
            add = add.priority(metric);
        }

        add.execute().await?;
        Ok(())
    }

    /// Adds a simple IPv6 route with an optional gateway.
    pub async fn route_add6(&self, r: Route6) -> Result<()> {
        let link = self
            .handle()
            .link()
            .get()
            .match_name(r.link.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(r.link))?;

        let id = link.header.index;

        let mut add = self
            .handle()
            .route()
            .add()
            .v6()
            .destination_prefix(r.dst, r.prefix_len)
            .output_interface(id);

        if let Some(rtr) = r.rtr {
            add = add.gateway(rtr);
        }

        if r.on_link {
            add = add.scope(RouteScope::Link);
        }

        if let Some(table) = r.table {
            add = add.table_id(table);
        }

        if let Some(metric) = r.metric {
            add = add.priority(metric);
        }

        add.execute().await?;
        Ok(())
    }

    /// Deletes a simple IPv4 route with an optional gateway.
    pub async fn route_del4(&self, r: Route4) -> Result<()> {
        let link = self
            .handle()
            .link()
            .get()
            .match_name(r.link.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(r.link))?;

        let id = link.header.index;

        let mut add = self
            .handle()
            .route()
            .add()
            .v4()
            .destination_prefix(r.dst, r.prefix_len)
            .output_interface(id);

        if let Some(rtr) = r.rtr {
            add = add.gateway(rtr);
        }

        if r.on_link {
            add = add.scope(RouteScope::Link);
        }

        if let Some(table) = r.table {
            add = add.table_id(table);
        }

        if let Some(metric) = r.metric {
            add = add.priority(metric);
        }

        self.handle()
            .route()
            .del(add.message_mut().clone())
            .execute()
            .await?;
        Ok(())
    }

    /// Deletes a simple IPv6 route with an optional gateway.
    pub async fn route_del6(&self, r: Route6) -> Result<()> {
        let link = self
            .handle()
            .link()
            .get()
            .match_name(r.link.clone())
            .execute()
            .try_next()
            .await?
            .ok_or(Error::LinkNotFound(r.link))?;

        let id = link.header.index;

        let mut add = self
            .handle()
            .route()
            .add()
            .v6()
            .destination_prefix(r.dst, r.prefix_len)
            .output_interface(id);

        if let Some(rtr) = r.rtr {
            add = add.gateway(rtr);
        }

        if r.on_link {
            add = add.scope(RouteScope::Link);
        }

        if let Some(table) = r.table {
            add = add.table_id(table);
        }

        if let Some(metric) = r.metric {
            add = add.priority(metric);
        }

        self.handle()
            .route()
            .del(add.message_mut().clone())
            .execute()
            .await?;
        Ok(())
    }
}
