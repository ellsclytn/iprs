mod context;
mod error;
mod interface;

use clap::{arg, Parser};
use context::Ctx;
use error::Result;
use interface::Interface;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use std::net::IpAddr;
use std::process;
use std::str::FromStr;

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
        Err(e) => {
            if let Ok(ipv4) = Ipv4Net::from_str(ip) {
                IpNet::V4(ipv4)
            } else if let Ok(ipv6) = Ipv6Net::from_str(ip) {
                IpNet::V6(ipv6)
            } else {
                return Err(e.into());
            }
        }
    };

    Ok(parsed_ip)
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut ctx = Ctx::new(std::io::stdout().lock(), std::io::stderr().lock());

    let ip_inputs = match &args.ip {
        Some(ips) => ips,
        None => {
            ctx.ewriteln("No IP subnet supplied")?;
            process::exit(1);
        }
    };

    for ip_input in ip_inputs.iter() {
        let interface = match parse_ip(ip_input) {
            Ok(ip) => ip,
            Err(_) => {
                ctx.writeln("-[int-ipv4 : {ip_input}] - 0\n")?;
                ctx.writeln("-[ERR : Unable to retrieve interface information]\n\n-")?;
                continue;
            }
        };

        if args.random && args.split.is_none() {
            ctx.ewriteln("-[ERR : --random requires --split]\n\n-")?;
            process::exit(1);
        }

        if let Some(split) = args.split {
            if args.random {
                interface.random_split(&mut ctx, split)?;
            } else {
                interface.split(&mut ctx, split)?;
            }
        } else {
            interface.summarize(&mut ctx)?;
        }

        ctx.writeln("\n-")?;
    }

    Ok(())
}
