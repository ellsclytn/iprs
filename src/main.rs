mod interface;

use clap::{arg, Parser};
use interface::Summary;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use std::net::IpAddr;
use std::process;
use std::str::FromStr;

#[derive(Parser)]
struct Cli {
    #[arg(trailing_var_arg(true))]
    ip: Option<Vec<String>>,
}

fn parse_ip(ip: &str) -> Result<IpNet, Box<dyn std::error::Error>> {
    let parsed_ip = match IpAddr::from_str(ip) {
        Ok(IpAddr::V4(ipv4)) => IpNet::V4(Ipv4Net::new(ipv4, 32)?),
        Ok(IpAddr::V6(ipv6)) => IpNet::V6(Ipv6Net::new(ipv6, 128)?),
        Err(e) => {
            if let Ok(ipv4) = Ipv4Net::from_str(ip) {
                IpNet::V4(ipv4)
            } else if let Ok(ipv6) = Ipv6Net::from_str(ip) {
                IpNet::V6(ipv6)
            } else {
                return Err(Box::new(e));
            }
        }
    };

    Ok(parsed_ip)
}

fn main() {
    let args = Cli::parse();

    let ip_inputs = match &args.ip {
        Some(ips) => ips,
        None => {
            eprintln!("No IP subnet supplied");
            process::exit(1);
        }
    };

    for ip_input in ip_inputs.iter() {
        let interface = match parse_ip(ip_input) {
            Ok(ip) => ip,
            Err(_) => {
                println!("-[int-ipv4 : {ip_input}] - 0\n");
                println!("-[ERR : Unable to retrieve interface information]\n\n-");
                continue;
            }
        };

        match interface {
            IpNet::V4(ipv4) => {
                println!("{}", ipv4.summarize());
            }
            IpNet::V6(ipv6) => {
                println!("{}", ipv6.summarize());
            }
        }

        println!("\n-");
    }
}
