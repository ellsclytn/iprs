use std::net::Ipv4Addr;

use crate::subnet::Subnet;
use ipnet::Ipv4Net;

fn ipv4_addr_to_u32(ip: Ipv4Addr) -> u32 {
    u32::from_be_bytes(ip.octets())
}

fn u32_to_ipv4_addr(ip: u32) -> Ipv4Addr {
    Ipv4Addr::from(ip.to_be_bytes())
}
pub struct Ipv4Subnet {
    parsed_subnet: Ipv4Net,
}

impl Ipv4Subnet {
    pub fn new(parsed_subnet: Ipv4Net) -> Self {
        Self { parsed_subnet }
    }
}

impl Subnet for Ipv4Subnet {
    fn print(&self) -> () {
        println!("-[ipv4 : {}] - 0\n", self.parsed_subnet);
        println!("[CIDR]");
        let host_address_decimal = ipv4_addr_to_u32(self.parsed_subnet.addr());
        println!("{0: <25}- {1}", "Host address", self.parsed_subnet.addr());
        println!(
            "{0: <25}- {1}",
            "Host address (decimal)", host_address_decimal
        );
        println!(
            "{0: <25}- {1:X}",
            "Host address (hex)", host_address_decimal
        );
        println!(
            "{0: <25}- {1}",
            "Network address",
            self.parsed_subnet.network()
        );
        println!(
            "{0: <25}- {1}",
            "Network mask",
            self.parsed_subnet.netmask()
        );
        println!(
            "{0: <25}- {1}",
            "Network mask (bits)",
            self.parsed_subnet.prefix_len()
        );
        let netmask_decimal = ipv4_addr_to_u32(self.parsed_subnet.netmask());
        println!("{0: <25}- {1:X}", "Network mask (hex)", netmask_decimal);
        println!(
            "{0: <25}- {1}",
            "Broadcast address",
            self.parsed_subnet.broadcast()
        );
        let cisco_wildcard = u32_to_ipv4_addr(!netmask_decimal);
        println!("{0: <25}- {1}", "Cisco wildcard", cisco_wildcard);
        let addresses_in_network = 2u32.pow(32 - self.parsed_subnet.prefix_len() as u32);
        println!(
            "{0: <25}- {1}",
            "Addresses in network", addresses_in_network
        );
        let network_range_start = self.parsed_subnet.network();
        let network_range_end = self.parsed_subnet.broadcast();
        println!(
            "{0: <25}- {1}",
            "Network range",
            format!("{} - {}", network_range_start, network_range_end)
        );

        let usable_range_start = if self.parsed_subnet.prefix_len() < 31 {
            u32_to_ipv4_addr(ipv4_addr_to_u32(network_range_start) + 1)
        } else {
            network_range_start
        };
        let usable_range_end = if self.parsed_subnet.prefix_len() < 31 {
            u32_to_ipv4_addr(ipv4_addr_to_u32(network_range_end) - 1)
        } else {
            network_range_end
        };
        println!(
            "{0: <25}- {1}",
            "Usable range",
            format!("{} - {}", usable_range_start, usable_range_end)
        );
    }
}
