use num_traits::{One, PrimInt, Zero};
use std::fmt;
use std::io::Write;

use crate::{
    error::{Error, Result},
    rng::RandomRangeGenerator,
    Ctx,
};

pub trait NetworkPrimitive: PrimInt {
    const BITS: u8;
    const MAX: Self;
}

impl NetworkPrimitive for u32 {
    const BITS: u8 = 32;
    const MAX: Self = u32::MAX;
}

impl NetworkPrimitive for u128 {
    const BITS: u8 = 128;
    const MAX: Self = u128::MAX;
}

pub trait NetworkCore {
    type Address: fmt::Display + Copy + Into<Self::Primitive> + From<Self::Primitive>;
    type Primitive: NetworkPrimitive;

    fn addr(&self) -> Self::Address;
    fn prefix_len(&self) -> u8;
    fn broadcast(&self) -> Self::Address;
    fn subnets(&self, prefix_len: u8) -> Result<impl Iterator<Item = Self>>
    where
        Self: Sized;

    fn addr_to_primitive(&self, addr: Self::Address) -> Self::Primitive {
        addr.into()
    }

    fn primitive_to_addr(&self, prim: Self::Primitive) -> Self::Address {
        prim.into()
    }
}

pub trait NetworkDisplay: NetworkCore + fmt::Display {
    const IP_VERSION: &'static str;
    const FORMAT_WIDTH: usize;

    fn format_attribute<T: fmt::Display>(name: &str, value: T) -> String {
        format!("{name: <24}- {value}")
    }

    fn split<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>, mask: u8) -> Result<()>
    where
        Self: Sized,
        Self::Address: fmt::Display,
    {
        ctx.writeln(format!("-[{} : {}] - 0\n", Self::IP_VERSION, self))?;
        ctx.writeln("[Split network]".to_string())?;

        match self.subnets(mask) {
            Ok(subnets) => {
                for subnet in subnets {
                    ctx.writeln(format!(
                        "Network - {:<width$} - {}",
                        subnet.addr(),
                        subnet.broadcast(),
                        width = Self::FORMAT_WIDTH
                    ))?;
                }
                Ok(())
            }
            Err(_) => Err(Error::SplitSmallerThanPrefixLen(mask, self.prefix_len())),
        }
    }

    fn summarize_random_split<W: Write, E: Write, R: RandomRangeGenerator<Self::Primitive>>(
        &self,
        ctx: &mut Ctx<W, E>,
        split: u8,
        rng: &mut R,
    ) -> Result<()>
    where
        Self: Sized + NetworkSummarize<W, E>,
    {
        let address = self.generate_random_split(split, rng)?;
        address.summarize(ctx)?;
        Ok(())
    }

    fn generate_random_split<R: RandomRangeGenerator<Self::Primitive>>(
        &self,
        split: u8,
        rng: &mut R,
    ) -> Result<Self>
    where
        Self: Sized,
    {
        if split <= self.prefix_len() {
            return Err(Error::SplitSmallerThanPrefixLen(split, self.prefix_len()));
        } else if split > Self::Primitive::BITS {
            return Err(Error::SplitTooBig(Self::Primitive::BITS, split));
        }

        let random_number = rng.random_range(Self::Primitive::zero()..Self::Primitive::MAX);
        let split_mask = !((Self::Primitive::one() << (Self::Primitive::BITS - split) as usize)
            - Self::Primitive::one());

        if self.prefix_len() == 0 {
            let random_address = random_number & split_mask;
            return Self::from_addr_prefix(self.primitive_to_addr(random_address), split);
        }

        let supernet_mask = (Self::Primitive::one()
            << (Self::Primitive::BITS - self.prefix_len()) as usize)
            - Self::Primitive::one();
        let new_address = (self.addr_to_primitive(self.addr()) & !supernet_mask)
            | (random_number & supernet_mask);
        let new_prefix = new_address & split_mask;

        Self::from_addr_prefix(self.primitive_to_addr(new_prefix), split)
    }

    /// Creates a new address from an Address and prefix length.
    fn from_addr_prefix(addr: Self::Address, prefix: u8) -> Result<Self>
    where
        Self: Sized;
}

pub trait NetworkSummarize<W: Write, E: Write>: NetworkCore {
    fn summarize(&self, ctx: &mut Ctx<W, E>) -> Result<()>;
}
