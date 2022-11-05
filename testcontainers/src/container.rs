pub struct Container<'d, I: Image> {
    id: String,
    // docker_client: Box<dyn DockerAsync>,
    // image: RunnableImage<I>,
    // command: Command,
    // /// Tracks the lifetime of the client to make sure the container is dropped before the client.
    // client_lifetime: PhantomData<&'d ()>,
}

impl<'d, I> Container<'d, I>
where
    I: Image,
{
    /// Returns the id of this container.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Starts the container.
    pub async fn start(&self) {
        self.docker_client.start(&self.id).await
    }

    pub async fn stop(&self) {
        log::debug!("Stopping docker container {}", self.id);

        self.docker_client.stop(&self.id).await
    }

    pub async fn rm(self) {
        log::debug!("Deleting docker container {}", self.id);

        self.docker_client.rm(&self.id).await
    }

    async fn drop_async(&self) {
        match self.command {
            env::Command::Remove => self.docker_client.rm(&self.id).await,
            env::Command::Keep => {}
        }
    }
}

impl<'d, I> fmt::Debug for Container<'d, I>
where
    I: fmt::Debug + Image,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Container")
            .field("id", &self.id)
            .field("image", &self.image)
            .finish()
    }
}
impl<'d, I> Drop for Container<'d, I>
where
    I: Image,
{
    fn drop(&mut self) {
        block_on(self.drop_async())
    }
}
