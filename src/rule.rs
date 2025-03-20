//! Simple functions to add and delete rules (for policy routing).

use crate::{Connection, Error, Result};

pub use netlink_packet_route::rule::RuleAction;

use std::net::{Ipv4Addr, Ipv6Addr};

use netlink_packet_route::rule::RuleFlag;
use rtnetlink::RuleAddRequest;

/// A rule entry.
#[derive(Clone, Debug)]
pub struct Rule<T: Clone> {
    /// Whether to invert the matching criteria.
    pub invert: bool,
    /// Firewall mark to match against.
    pub fwmark: Option<u32>,
    /// Destination prefix to match against.
    pub dst: Option<(T, u8)>,
    /// Source prefix to match against.
    pub src: Option<(T, u8)>,
    /// Action to perform.
    pub action: RuleAction,
    /// Routing table to use if `RuleAction::ToTable` is selected.
    pub table: u32,
}

impl Rule<()> {
    pub async fn add(self, c: &Connection) -> Result<()> {
        let add = self.prepare_add(c);

        if self.dst.is_some() || self.src.is_some() {
            return Err(Error::PrefixesDisallowed);
        }

        add.execute().await?;
        Ok(())
    }

    pub async fn del(self, c: &Connection) -> Result<()> {
        let mut add = self.prepare_add(c);

        if self.dst.is_some() || self.src.is_some() {
            return Err(Error::PrefixesDisallowed);
        }

        c.handle()
            .rule()
            .del(add.message_mut().clone())
            .execute()
            .await?;
        Ok(())
    }
}

impl Rule<Ipv4Addr> {
    pub async fn add(self, c: &Connection) -> Result<()> {
        let mut add = self.prepare_add(c).v4();

        if let Some(dst) = self.dst {
            add = add.destination_prefix(dst.0, dst.1);
        }
        if let Some(src) = self.src {
            add = add.source_prefix(src.0, src.1)
        }

        add.execute().await?;
        Ok(())
    }

    pub async fn del(self, c: &Connection) -> Result<()> {
        let mut add = self.prepare_add(c).v4();

        if let Some(dst) = self.dst {
            add = add.destination_prefix(dst.0, dst.1);
        }
        if let Some(src) = self.src {
            add = add.source_prefix(src.0, src.1)
        }

        c.handle()
            .rule()
            .del(add.message_mut().clone())
            .execute()
            .await?;
        Ok(())
    }
}

impl Rule<Ipv6Addr> {
    pub async fn add(self, c: &Connection) -> Result<()> {
        let mut add = self.prepare_add(c).v6();

        if let Some(dst) = self.dst {
            add = add.destination_prefix(dst.0, dst.1);
        }
        if let Some(src) = self.src {
            add = add.source_prefix(src.0, src.1)
        }

        add.execute().await?;
        Ok(())
    }

    pub async fn del(self, c: &Connection) -> Result<()> {
        let mut add = self.prepare_add(c).v6();

        if let Some(dst) = self.dst {
            add = add.destination_prefix(dst.0, dst.1);
        }
        if let Some(src) = self.src {
            add = add.source_prefix(src.0, src.1)
        }

        c.handle()
            .rule()
            .del(add.message_mut().clone())
            .execute()
            .await?;
        Ok(())
    }
}

impl<T: Clone> Rule<T> {
    fn prepare_add(&self, c: &Connection) -> RuleAddRequest {
        let mut add = c.handle().rule().add().action(self.action);

        if self.invert {
            add.message_mut().header.flags.push(RuleFlag::Invert);
        }

        if let Some(fwmark) = self.fwmark {
            add = add.fw_mark(fwmark)
        }

        if self.action == RuleAction::ToTable {
            add = add.table_id(self.table)
        }

        add
    }
}
