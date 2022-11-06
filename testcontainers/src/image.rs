use super::wait::WaitFor;
use std::collections::BTreeMap;

/// Represents a port mapping between a local port and the internal port of a container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Port {
    pub host: u16,
    pub container: u16,
}

// impl From<(u16, u16)> for Port {
//     fn from((local, internal): (u16, u16)) -> Self {
//         Port { local, internal }
//     }
// }

// todo: add wait conditions etc.

/// DockerImage describes a docker container image
///
/// https://pkg.go.dev/github.com/testcontainers/testcontainers-go#ContainerRequest
///
#[must_use]
#[derive(Default, Debug)]
// pub struct DockerImage<I: Image> {
pub struct DockerImage {
    // todo: add more configuration settings from the go implementation
    pub image: String,
    pub image_tag: Option<String>,
    pub entrypoint: Vec<String>,
    pub exposed_ports: Vec<Port>,
    pub port_mapping: Vec<Port>,
    pub cmd: Vec<String>,
    pub labels: BTreeMap<String, String>,
    pub tmpfs: BTreeMap<String, String>,
    pub registry_credentials: Option<String>,
    pub hostname: Vec<String>,
    pub extra_hosts: Vec<String>,
    pub container_name: Option<String>,
    pub networks: Vec<String>,
    pub network_aliases: BTreeMap<String, Vec<String>>,
    pub network_mode: Option<String>,
    pub env_vars: BTreeMap<String, String>,
    pub volumes: BTreeMap<String, String>,
    pub privileged: bool,
    pub shm_size: Option<i64>,
    pub waiting_for: Vec<WaitFor>,
}

impl DockerImage {
    pub fn descriptor(&self) -> String {
        format!(
            "{}:{}",
            self.image,
            self.image_tag.as_deref().unwrap_or("latest")
        )
    }
}

impl DockerImage {
    pub fn new(image: impl Into<String>) -> DockerImage {
        DockerImage {
            image: image.into(),
            ..Default::default()
        }
    }

    pub fn with_tag(self, tag: impl Into<String>) -> Self {
        Self {
            image_tag: Some(tag.into()),
            ..self
        }
    }

    pub fn with_container_name(self, name: impl Into<String>) -> Self {
        Self {
            container_name: Some(name.into()),
            ..self
        }
    }

    pub fn with_network(self, network: impl Into<String>) -> Self {
        let mut networks = self.networks;
        networks.push(network.into());
        Self { networks, ..self }
    }

    pub fn with_env_var(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let mut env_vars = self.env_vars;
        env_vars.insert(key.into(), value.into());
        Self { env_vars, ..self }
    }

    pub fn with_volume(self, src: impl Into<String>, dest: impl Into<String>) -> Self {
        let mut volumes = self.volumes;
        volumes.insert(src.into(), dest.into());
        Self { volumes, ..self }
    }

    // pub fn with_exposed_port(self, container_port: u16) -> Self {
    //     let mut ports = self.exposed_ports;
    //     ports.push(Port {
    //         host: host_port,
    //         container: container_port,
    //     });

    //     Self {
    //         exposed_ports: ports,
    //         ..self
    //     }
    // }

    pub fn with_mapped_port(self, host_port: u16, container_port: u16) -> Self {
        let mut ports = self.exposed_ports;
        ports.push(Port {
            host: host_port,
            container: container_port,
        });

        Self {
            exposed_ports: ports,
            ..self
        }
    }

    pub fn with_privileged(self, privileged: bool) -> Self {
        Self { privileged, ..self }
    }

    pub fn with_shm_size(self, bytes: i64) -> Self {
        Self {
            shm_size: Some(bytes),
            ..self
        }
    }
}
