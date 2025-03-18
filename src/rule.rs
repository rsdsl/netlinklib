//! Simple functions to add and delete rules (for policy routing).

use crate::{Connection, Error, Result};

pub use netlink_packet_route::rule::RuleAction;

use netlink_packet_route::rule::{RuleAttribute, RuleFlag, RuleHeader, RuleMessage};
use rtnetlink::RuleAddRequest;

trait IpAddr46 {}

impl IpAddr46 for () {}
impl IpAddr46 for Ipv4Addr {}
impl IpAddr46 for Ipv6Addr {}

/// A rule entry.
#[derive(Debug)]
pub struct Rule<A: IpAddr46> {
    /// Whether to invert the matching criteria.
    pub invert: bool,
    /// Firewall mark to match against.
    pub fwmark: Option<u32>,
    /// Destination prefix to match against.
    pub dst: Option<(A, u8)>,
    /// Source prefix to match against.
    pub src: Option<(A, u8)>,
    /// Action to perform.
    pub action: RuleAction,
    /// Routing table to use if `RuleAction::ToTable` is selected.
    pub table: u32,
}

impl Rule<()> {
    fn addSrcDst(&self, rq: RuleAddRequest) -> RuleAddRequest {
        rq
    }
}

impl Rule<Ipv4Addr> {
    fn addSrcDst(&self, mut rq: RuleAddRequest) -> RuleAddRequest {
        rq = rq.v4();
        if let Some(dst) = self.dst {
            rq = rq.destination_prefix(dst.0, dst.1);
        }
        if let Some(src) = self.src {
            rq = rq.destination_prefix(src.0, src.1);
        }

        rq
    }
}

impl Rule<Ipv6Addr> {
    fn addSrcDst(&self, mut rq: RuleAddRequest) -> RuleAddRequest {
        rq = rq.v6();
        if let Some(dst) = self.dst {
            rq = rq.destination_prefix(dst.0, dst.1);
        }
        if let Some(src) = self.src {
            rq = rq.destination_prefix(src.0, src.1);
        }

        rq
    }
}

impl<A: IpAddr46> Connection {
    /// Adds a rule entry.
    pub async fn rule_add(&self, r: Rule<A>) -> Result<()> {
        let mut add = self.handle().rule().add().action(r.action);

        if let Some(fwmark) = r.fwmark {
            add = add.fw_mark(fwmark);
        }
        if let Some(table) = r.table {
            add = add.table_id(table);
        }

        add = r.addSrcDst(add);

        add.message_mut().header.flags.push(RuleFlag::Invert);

        add.execute().await?;
        Ok(())
    }

    /// Deletes a rule entry.
    pub async fn rule_del(&self, r: Rule<A>) -> Result<()> {
        let mut add = self.handle().rule().add().action(r.action);

        if let Some(fwmark) = r.fwmark {
            add = add.fw_mark(fwmark);
        }
        if let Some(table) = r.table {
            add = add.table_id(table);
        }

        add = r.addSrcDst(add);

        add.message_mut().header.flags.push(RuleFlag::Invert);

        self.handle()
            .rule()
            .del(add.message_mut().clone())
            .execute()
            .await?;
        Ok(())
    }
}
