use super::client::DockerClient;
use super::image::DockerImage;
use bollard::Docker;
use futures::executor::block_on;
use std::{fmt, net};

// pub struct Container<'d, I: Image> {
pub struct Container<C>
// pub struct Container
where
    C: DockerClient,
{
    id: String,
    // client: Box<&dyn DockerClient>,
    // client: Box<dyn DockerClient<Client = _, Error = _>>,
    client: C,
    image: DockerImage,
    // image: RunnableImage<I>,
    // command: Command,
    // /// Tracks the lifetime of the client to make sure the container is dropped before the client.
    // client_lifetime: PhantomData<&'d ()>,
}

// impl<'d, I> Container<'d, I>
impl<C> Container<C>
// impl Container
where
    C: DockerClient,
    // I: Image,
{
    /// Returns the id of this container.
    pub async fn new(
        id: String,
        // client: impl DockerClient + 'static,
        client: C, // impl DockerClient + 'static,
        image: DockerImage,
        // command: env::Command,
    ) -> Self {
        let container = Self {
            id,
            client,
            image,
            // command,
            // client_lifetime: PhantomData,
        };
        // container.block_until_ready().await;
        container
    }

    async fn block_until_ready(&self) {
        log::debug!("Waiting for container {} to be ready", self.id);

        // for condition in self.image.ready_conditions() {
        //     match condition {
        //         WaitFor::StdOutMessage { message } => self
        //             .docker_client
        //             .stdout_logs(&self.id)
        //             .wait_for_message(&message)
        //             .await
        //             .unwrap(),
        //         WaitFor::StdErrMessage { message } => self
        //             .docker_client
        //             .stderr_logs(&self.id)
        //             .wait_for_message(&message)
        //             .await
        //             .unwrap(),
        //         WaitFor::Duration { length } => {
        //             tokio::time::sleep(length).await;
        //         }
        //         WaitFor::Healthcheck => loop {
        //             use HealthStatusEnum::*;

        //             let health_status = self
        //                 .docker_client
        //                 .inspect(&self.id)
        //                 .await
        //                 .state
        //                 .unwrap_or_else(|| panic!("Container state not available"))
        //                 .health
        //                 .unwrap_or_else(|| panic!("Health state not available"))
        //                 .status;

        //             match health_status {
        //                 Some(HEALTHY) => break,
        //                 None | Some(EMPTY) | Some(NONE) => {
        //                     panic!("Healthcheck not configured for container")
        //                 }
        //                 Some(UNHEALTHY) => panic!("Healthcheck reports unhealthy"),
        //                 Some(STARTING) => sleep(Duration::from_millis(100)).await,
        //             }
        //             panic!("Healthcheck for the container is not configured");
        //         },
        //         WaitFor::Nothing => {}
        //     }
        // }

        log::debug!("container {} is ready!", self.id);
    }

    /// Returns the id of this container.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Starts the container.
    pub async fn start(&self) -> Result<(), C::Error> {
        log::debug!("starting docker container {}", self.id);
        self.client.start(&self.id).await
    }

    /// Stops the container
    pub async fn stop(&self) -> Result<(), C::Error> {
        log::debug!("stopping docker container {}", self.id);
        self.client.stop(&self.id).await
    }

    /// Removes the container
    pub async fn rm(self) -> Result<(), C::Error> {
        log::debug!("removing docker container {}", self.id);
        self.client.rm(&self.id).await
    }

    /// Gets the host IP address of the container
    pub async fn host(&self) -> Result<net::IpAddr, C::Error> {
        self.client.host(&self.id).await
    }

    /// Get the mapped host IPv4 port for the given internal port
    pub async fn mapped_port_ipv4(&self, internal_port: u16) -> Result<Option<u16>, C::Error> {
        let ports = self.client.ports(&self.id).await?;
        Ok(ports.mapped_port_ipv4(internal_port))
    }

    /// Get the mapped host IPv6 port for the given internal port
    pub async fn mapped_port_ipv6(&self, internal_port: u16) -> Result<Option<u16>, C::Error> {
        let ports = self.client.ports(&self.id).await?;
        Ok(ports.mapped_port_ipv6(internal_port))
    }

    /// Drops and removes the container
    async fn drop_async(&self) {
        if let Err(err) = self.client.rm(&self.id).await {
            log::error!("failed to remove docker container {}", self.id);
        }
        // match self.command {
        //     env::Command::Remove => self.docker_client.rm(&self.id).await,
        //     env::Command::Keep => {}
        // }
    }
}

// impl<'d, I> fmt::Debug for Container<'d, I>
// where
//     I: fmt::Debug + Image,
impl<C> fmt::Debug for Container<C>
// impl fmt::Debug for Container
where
    C: DockerClient,
    // I: fmt::Debug + Image,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Container")
            .field("id", &self.id)
            .field("image", &self.image.descriptor())
            .finish()
    }
}

// impl<'d, I> Drop for Container<'d, I>
impl<C> Drop for Container<C>
// impl Drop for Container
where
    C: DockerClient,
    // I: Image,
{
    fn drop(&mut self) {
        block_on(self.drop_async())
    }
}
