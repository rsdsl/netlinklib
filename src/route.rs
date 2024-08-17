//! Simple functions to add and delete routes.

use crate::{Connection, Error, Result};

use std::net::{Ipv4Addr, Ipv6Addr};

use futures::{future, TryStreamExt};
use netlink_packet_route::route::{RouteAttribute, RouteMessage, RouteScope};
use rtnetlink::IpVersion;

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
    pub async fn route_add4(
        &self,
        dst: Ipv4Addr,
        prefix_len: u8,
        rtr: Option<Ipv4Addr>,
        link: String,
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

        let mut add = self
            .handle()
            .route()
            .add()
            .v4()
            .destination_prefix(dst, prefix_len)
            .output_interface(id);

        if let Some(rtr) = rtr {
            add = add.gateway(rtr);
        } else {
            add = add.scope(RouteScope::Link);
        }

        add.execute().await?;
        Ok(())
    }

    /// Adds a simple IPv6 route with an optional gateway.
    pub async fn route_add6(
        &self,
        dst: Ipv6Addr,
        prefix_len: u8,
        rtr: Option<Ipv6Addr>,
        link: String,
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

        let mut add = self
            .handle()
            .route()
            .add()
            .v6()
            .destination_prefix(dst, prefix_len)
            .output_interface(id);

        if let Some(rtr) = rtr {
            add = add.gateway(rtr);
        } else {
            add = add.scope(RouteScope::Link);
        }

        add.execute().await?;
        Ok(())
    }
}
