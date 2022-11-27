use super::container::Container;
use super::image::DockerImage;
use super::logs::LogStream;
use super::ports::Ports;
use async_trait::async_trait;
use std::net;

#[async_trait]
pub trait DockerClient
where
    Self: Sized,
{
    type Client;
    type Error: std::error::Error;

    fn native(&self) -> &Self::Client;
    fn stdout_logs(&self, id: &str) -> LogStream<'_>;
    fn stderr_logs(&self, id: &str) -> LogStream<'_>;

    async fn create<I: Into<DockerImage> + Send>(
        &self,
        image: I,
    ) -> Result<Container<Self>, Self::Error>;

    async fn host(&self, id: &str) -> Result<net::IpAddr, Self::Error>;
    async fn ports(&self, id: &str) -> Result<Ports, Self::Error>;
    async fn rm(&self, id: &str) -> Result<(), Self::Error>;
    async fn stop(&self, id: &str) -> Result<(), Self::Error>;
    async fn start(&self, id: &str) -> Result<(), Self::Error>;

    // async fn inspect(&self, id: &str) -> ContainerInspectResponse;
}

pub mod bollard {
    use super::{Container, DockerClient, DockerImage, LogStream, Ports};
    use async_trait::async_trait;
    use bollard::container::LogsOptions;
    use color_eyre::eyre;
    use futures::{StreamExt, TryStreamExt};
    use std::sync::Arc;
    use std::{fmt, io, net};

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("missing host")]
        MissingHost,

        #[error("failed to parse address {addr}")]
        ParseAddr {
            addr: String,
            source: std::net::AddrParseError,
        },

        #[error("failed to connect to the docker daemon")]
        Connection(#[source] bollard::errors::Error),

        #[error(transparent)]
        Bollard(#[from] bollard::errors::Error),
    }

    #[derive(Clone)]
    pub struct Client {
        inner: Arc<bollard::Docker>,
        id: Option<String>,
    }

    impl fmt::Debug for Client {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Client").field("id", &self.id).finish()
        }
    }

    impl Client {
        pub async fn new() -> Result<Self, Error> {
            let client =
                bollard::Docker::connect_with_local_defaults().map_err(Error::Connection)?;
            // bollard::Docker::connect_with_http_defaults().map_err(Error::Connection)?;
            let inner = Arc::new(client);
            let id = inner.info().await.ok().and_then(|info| info.id);
            Ok(Self { inner, id })
        }

        fn logs(&self, id: &str, options: LogsOptions<String>) -> LogStream<'_> {
            let stream = self
                .inner
                .logs(&id, Some(options))
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
                .map(|chunk| {
                    let bytes = chunk?.into_bytes();
                    Ok(String::from_utf8_lossy(bytes.as_ref()).to_string())
                });
            LogStream::new(stream)
        }
    }

    // impl std::ops::Deref for Client {
    //     type Target = bollard::Docker;

    //     fn deref(&self) -> &Self::Target {
    //         &self.inner
    //     }
    // }

    // impl std::ops::DerefMut for Client {
    //     fn deref_mut(&mut self) -> &mut Self::Target {
    //         &mut self.inner
    //     }
    // }

    #[async_trait]
    impl DockerClient for Client {
        type Client = bollard::Docker;
        type Error = Error;

        async fn create<I: Into<DockerImage> + Send>(
            &self,
            image: I,
        ) -> Result<Container<Self>, Self::Error> {
            use bollard::container::{Config, CreateContainerOptions};
            use bollard::models::{HostConfig, PortBinding};
            use std::collections::HashMap;

            let image = image.into();

            let volumes: HashMap<String, HashMap<(), ()>> = image
                .volumes
                .iter()
                .map(|(orig, dest)| (format!("{}:{}", orig, dest), HashMap::new()))
                .collect();

            let mut exposed_ports: HashMap<String, HashMap<(), ()>> = HashMap::new();
            // let mut exposed_ports = HashMap::new();
            let mut port_bindings = HashMap::new();
            for port in &image.exposed_ports {
                let proto_port = format!("{}/tcp", port.container);
                exposed_ports.insert(proto_port.clone(), HashMap::new());
                port_bindings.insert(
                    proto_port,
                    None::<Vec<PortBinding>>,
                    // Some(vec![PortBinding {
                    //     host_ip: Some(String::from("127.0.0.1")),
                    //     host_port: Some(port.host.to_string()),
                    // }]),
                );
            }

            // let exposed_ports: HashMap<String, Option<Vec<HashMap<(), ()>> =
            //     HashMap::from_iter(vec![("80".to_string(), HashMap::new())]);

            let mut host_config = HostConfig {
                shm_size: image.shm_size,
                // port_bindings: Some(port_bindings),
                ..Default::default()
            };

            // let exposed_ports: HashMap<String, HashMap<(), ()>> =
            //     HashMap::from_iter(vec![("80".to_string(), HashMap::new())]);

            let mut config: Config<String> = Config {
                image: Some(image.descriptor()),
                cmd: Some(image.cmd.clone()),
                exposed_ports: Some(exposed_ports),
                // env: Some(image.env),
                // volumes: Some(image.volumes),
                entrypoint: Some(image.entrypoint.clone()),
                volumes: Some(volumes),
                host_config: Some(host_config),
                ..Default::default()
            };

            // // create network and add it to container creation
            // if let Some(network) = image.network() {
            //     config.host_config = config.host_config.map(|mut host_config| {
            //         host_config.network_mode = Some(network.to_string());
            //         host_config
            //     });
            //     // if self.create_network_if_not_exists(network).await {
            //     //     let mut guard = self
            //     //         .inner
            //     //         .created_networks
            //     //         .write()
            //     //         .expect("'failed to lock RwLock'");
            //     //     guard.push(network.clone());
            //     // }
            // }

            let create_options: Option<CreateContainerOptions<String>> = image
                .container_name
                .as_ref()
                .map(|name| CreateContainerOptions {
                    name: name.to_owned(),
                });

            // pull the image first
            use bollard::image::CreateImageOptions;
            let pull_options = Some(CreateImageOptions {
                from_image: image.descriptor(),
                ..Default::default()
            });
            let mut pulling = self.inner.create_image(pull_options, None, None);
            log::debug!("Pulling docker container {}", image.descriptor());
            while let Some(result) = pulling.next().await {
                if let Err(err) = result {
                    return Err(err.into());
                }
            }
            log::debug!("Pulled docker container {}", image.descriptor());

            let container = self.inner.create_container(create_options, config).await?;
            // match container {
            //         // Ok(container) => container.id,
            //         Err(bollard::errors::Error::DockerResponseServerError {
            //             status_code: 404,
            //             ..
            //         }) => {
            //             // image not found locally, pull first
            //             {
            //                 use bollard::image::CreateImageOptions;
            //                 let pull_options = Some(CreateImageOptions {
            //                     from_image: image.descriptor(),
            //                     ..Default::default()
            //                 });
            //                 let mut pulling = self.inner.create_image(pull_options, None, None);
            //                 while let Some(result) = pulling.next().await {
            //                     if
            //                     // if result.is_err() {
            //                     //     result.unwrap();
            //                     // }
            //                 }
            //             }
            //             // try again
            //             self.create_container(create_options, config)
            //                 .await
            //                 .unwrap()
            //                 .id
            //         }
            //         Err(err) => return
            //     }

            // let container_id = {
            //     match container {
            //         Ok(container) => container.id,
            //         Err(bollard::errors::Error::DockerResponseServerError {
            //             status_code: 404,
            //             ..
            //         }) => {
            //             // image not found locally, pull first
            //             {
            //                 use bollard::image::CreateImageOptions;
            //                 let pull_options = Some(CreateImageOptions {
            //                     from_image: image.descriptor(),
            //                     ..Default::default()
            //                 });
            //                 let mut pulling = self.inner.create_image(pull_options, None, None);
            //                 while let Some(result) = pulling.next().await {
            //                     if
            //                     // if result.is_err() {
            //                     //     result.unwrap();
            //                     // }
            //                 }
            //             }
            //             // try again
            //             self.create_container(create_options, config)
            //                 .await
            //                 .unwrap()
            //                 .id
            //         }
            //         Err(err) => panic!("{}", err),
            //     }
            // };

            // let container_id = created_container.id;
            // let container = Container::new(container_id, self.clone(), image).await;
            Ok(Container::new(container.id, self.clone(), image).await)
        }

        fn native(&self) -> &Self::Client {
            &self.inner
        }

        fn stdout_logs(&self, id: &str) -> LogStream<'_> {
            self.logs(
                id,
                LogsOptions {
                    follow: true,
                    stdout: true,
                    tail: "all".to_string(),
                    ..Default::default()
                },
            )
        }

        fn stderr_logs(&self, id: &str) -> LogStream<'_> {
            self.logs(
                id,
                LogsOptions {
                    follow: true,
                    stderr: true,
                    tail: "all".to_string(),
                    ..Default::default()
                },
            )
        }

        async fn host(&self, id: &str) -> Result<net::IpAddr, Self::Error> {
            let inspect = self.inner.inspect_container(id, None).await?;
            let addr = inspect
                .network_settings
                .and_then(|network| network.ip_address)
                .ok_or(Self::Error::MissingHost)?;
            addr.parse()
                .map_err(|err| Self::Error::ParseAddr { addr, source: err })
        }

        async fn ports(&self, id: &str) -> Result<Ports, Self::Error> {
            let inspect = self.inner.inspect_container(id, None).await?;
            log::debug!("network settings: {:?}", inspect.network_settings);
            let ports: Ports = inspect
                .network_settings
                .unwrap_or_default()
                .ports
                .unwrap_or_default()
                .into();
            Ok(ports)
            // .unwrap_or_default()
        }

        // async fn inspect(&self, id: &str) -> Result<ContainerInspectResponse, Self::Error> {
        //     Ok(self.inner.inspect_container(id, None).await?)
        // }

        async fn rm(&self, id: &str) -> Result<(), Self::Error> {
            Ok(self
                .inner
                .remove_container(
                    id,
                    Some(bollard::container::RemoveContainerOptions {
                        force: true,
                        v: true,
                        ..Default::default()
                    }),
                )
                .await?)
        }

        async fn stop(&self, id: &str) -> Result<(), Self::Error> {
            Ok(self.inner.stop_container(id, None).await?)
        }

        async fn start(&self, id: &str) -> Result<(), Self::Error> {
            Ok(self.inner.start_container::<String>(id, None).await?)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{Client, DockerClient};
        use color_eyre::eyre;
        use pretty_assertions::{assert_eq, assert_ne};

        #[tokio::test(flavor = "multi_thread")]
        async fn get_native_client() -> eyre::Result<()> {
            let concrete: Client = Client::new().await?;
            // let client: Box<&dyn DockerClient<Client = _, Error = _>> = Box::new(&concrete);
            let native: &bollard::Docker = concrete.native();
            assert!(std::ptr::eq(&*concrete.inner, native));
            Ok(())
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn expose_all_ports_by_default() -> eyre::Result<()> {
            let client = Client::new().await?;
            // let docker = Http::new();
            // let image = HelloWorld::default();
            // let container = client.run(image).await?;

            // // inspect volume and env
            // let container_details = inspect(&docker.inner.bollard, container.id()).await;
            // assert_that!(container_details.host_config.unwrap().publish_all_ports)
            //     .is_equal_to(Some(true));
            Ok(())
        }
    }
}
