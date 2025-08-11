mod context;
mod error;
mod interface;
mod rng;

use clap::{arg, Parser};
use context::Ctx;
use error::Result;
use interface::Interface;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use std::io::Write;
use std::net::IpAddr;
use std::process;
use std::str::FromStr;

use crate::error::Error;

#[derive(Parser)]
struct Cli {
    #[arg(trailing_var_arg(true))]
    ip: Option<Vec<String>>,
    #[arg(short, long)]
    random: bool,
    #[arg(short, long)]
    split: Option<u8>,
}

fn parse_ip(ip: &str) -> Result<IpNet> {
    let parsed_ip = match IpAddr::from_str(ip) {
        Ok(IpAddr::V4(ipv4)) => IpNet::V4(Ipv4Net::new(ipv4, 32)?),
        Ok(IpAddr::V6(ipv6)) => IpNet::V6(Ipv6Net::new(ipv6, 128)?),
        Err(_) => {
            if let Ok(ipv4) = Ipv4Net::from_str(ip) {
                IpNet::V4(ipv4)
            } else if let Ok(ipv6) = Ipv6Net::from_str(ip) {
                IpNet::V6(ipv6)
            } else {
                return Err(Error::AddrParse(ip.to_string()));
            }
        }
    };

    Ok(parsed_ip)
}

fn run<W: Write, E: Write>(ctx: &mut Ctx<W, E>, args: Cli) -> Result<()> {
    let ip_inputs = match &args.ip {
        Some(ips) => ips,
        None => ctx.error_and_exit("No IP subnet supplied"),
    };

    for ip_input in ip_inputs.iter() {
        let interface = match parse_ip(ip_input) {
            Ok(ip) => ip,
            Err(e) => {
                ctx.error_without_exit(e)?;

                continue;
            }
        };

        if args.random && args.split.is_none() {
            ctx.error_and_exit("--random requires --split");
        }

        if let Some(split) = args.split {
            if args.random {
                interface.random_split(ctx, split)?;
            } else {
                match interface.split(ctx, split) {
                    Ok(()) => {}
                    Err(e) => {
                        ctx.error_without_exit(e)?;

                        continue;
                    }
                }
            }
        } else {
            interface.summarize(ctx)?;
        }

        ctx.writeln("\n-")?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut ctx = Ctx::new(std::io::stdout().lock(), std::io::stderr().lock());

    match run(&mut ctx, args) {
        Ok(_) => process::exit(ctx.errored as i32),
        Err(e) => ctx.error_and_exit(e),
    }
}
