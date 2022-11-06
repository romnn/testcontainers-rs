use bollard::models::{PortBinding, PortMap};
use std::collections::HashMap;
use std::net;

// /// PortMap describes the mapping of container ports to host ports, using the container's port-number and protocol as key in the format port>/<protocol>for example, 0/udp  If a container's port is mapped for multiple protocols, separate entries are added to the mapping table.
// // special-casing PortMap, cos swagger-codegen doesn't figure out this type
// pub type PortMap = HashMap<String, Option<Vec<PortBinding>>>;

fn parse_port(port: &str) -> u16 {
    port.parse()
        .unwrap_or_else(|e| panic!("Failed to parse {} as u16 because {}", port, e))
}

/// The exposed ports of a running container.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Ports {
    ipv4_mapping: HashMap<u16, u16>,
    ipv6_mapping: HashMap<u16, u16>,
}

impl Ports {
    /// Get the mapped host IPv4 port for the given internal port
    pub fn mapped_port_ipv4(&self, internal_port: u16) -> Option<u16> {
        self.ipv4_mapping.get(&internal_port).cloned()
    }

    /// Get the mapped host IPv6 port for the given internal port
    pub fn mapped_port_ipv6(&self, internal_port: u16) -> Option<u16> {
        self.ipv6_mapping.get(&internal_port).cloned()
    }
}

impl From<PortMap> for Ports {
    fn from(ports: PortMap) -> Self {
        dbg!(&ports);

        let mut ipv4_mapping = HashMap::new();
        let mut ipv6_mapping = HashMap::new();

        for (internal, external) in ports {
            // internal is of the form '8332/tcp', split off the protocol ...
            let internal_port = if let Some(internal) = internal.split('/').next() {
                parse_port(internal)
            } else {
                continue;
            };

            for binding in external.into_iter().flatten() {
                if let Some(external_port) = binding.host_port.as_ref() {
                    let external_port = parse_port(external_port);

                    // switch on the IP version
                    let mapping = match binding.host_ip.map(|ip| ip.parse()) {
                        Some(Ok(net::IpAddr::V4(_))) => {
                            // log::debug!(
                            //     "Registering IPv4 port mapping: {} -> {}",
                            //     internal_port,
                            //     external_port
                            // );
                            &mut ipv4_mapping
                        }
                        Some(Ok(net::IpAddr::V6(_))) => {
                            // log::debug!(
                            //     "Registering IPv6 port mapping: {} -> {}",
                            //     internal_port,
                            //     external_port
                            // );
                            &mut ipv6_mapping
                        }
                        Some(Err(_)) | None => continue,
                    };

                    mapping.insert(internal_port, external_port);
                } else {
                    continue;
                }
            }
        }

        Self {
            ipv4_mapping,
            ipv6_mapping,
        }
    }
}
