pub mod ipv4;
pub mod ipv6;

use std::io::Write;

use crate::context::Ctx;
use crate::error::Result;

use ipnet::IpNet;

pub trait Interface {
    fn summarize<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>) -> Result<()>;
    fn split<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>, split: u8) -> Result<()>;
    fn random_split<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>, split: u8) -> Result<()>;
}

impl Interface for IpNet {
    fn summarize<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>) -> Result<()> {
        match self {
            IpNet::V4(ipv4) => ipv4.summarize(ctx),
            IpNet::V6(ipv6) => ipv6.summarize(ctx),
        }
    }

    fn split<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>, mask: u8) -> Result<()> {
        match self {
            IpNet::V4(ipv4) => ipv4.split(ctx, mask),
            IpNet::V6(ipv6) => ipv6.split(ctx, mask),
        }
    }

    fn random_split<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>, split: u8) -> Result<()> {
        match self {
            IpNet::V4(ipv4) => ipv4.random_split(ctx, split),
            IpNet::V6(ipv6) => ipv6.random_split(ctx, split),
        }
    }
}
