#![allow(warnings)]

use color_eyre::eyre;
use testcontainers_rs::{
    client::{bollard::Client, DockerClient},
    Container, DockerImage,
};

// struct NginxContainer<C>
struct NginxContainer
// where
//     C: DockerClient,
{
    container: Container<Client>,
}

struct ContainerRequest {
    pub image: String,
}

// impl<C> NginxContainer<C> {
// impl As<DockerImage> for NginxContainer {

// }

impl NginxContainer {
    pub async fn new() -> eyre::Result<Self> {
        let client = Client::new().await?;
        let image = DockerImage::new("nginx").with_mapped_port(80, 80);
        let container = client.create(image).await?;
        Ok(Self { container })
    }

    pub async fn start(&self) -> eyre::Result<()> {
        self.container.start().await?;
        // req := testcontainers.ContainerRequest{
        // Image:        "nginx",
        // ExposedPorts: []string{"80/tcp"},
        // WaitingFor:   wait.ForHTTP("/"),
        // }

        // container, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
        //     ContainerRequest: req,
        //     Started:          true,
        // })
        Ok(())
    }

    pub async fn uri(&self) -> eyre::Result<reqwest::Url> {
        let host = self.container.host().await?;
        let port = self
            .container
            .mapped_port_ipv4(80)
            .await?
            .ok_or_else(|| eyre::eyre!("no mapped port"))?;

        // ip, err := container.Host(ctx)
        // if err != nil {
        // return nil, err
        // }

        // mappedPort, err := container.MappedPort(ctx, "80")
        // if err != nil {
        // return nil, err
        // }

        let uri = reqwest::Url::parse(&format!("http://{}:{}", host, port))?;
        Ok(uri)
        // uri := fmt.Sprintf("http://%s:%s", ip, mappedPort.Port())
        // reqwest::Uri::builder()
        //     // .scheme("https")
        //     // .authority("hyper.rs")
        //     // .path_and_query("/")
        //     .host(host)
        //     .port(0)
        //     .build()
        // .unwrap();
        // Ok("todo".to_string())
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    pretty_env_logger::init();

    let nginx = NginxContainer::new().await?;
    nginx.start().await?;

    let resp = reqwest::get(nginx.uri().await?).await?.text().await?;
    println!("{:#?}", resp);
    //
    // resp, err := http.Get(nginxC.URI)
    // if resp.StatusCode != http.StatusOK {
    // t.Fatalf("Expected status code %d. Got %d.", http.StatusOK, resp.StatusCode)
    // }

    // when the `NginxContainer` is dropped, the container is terminated
    Ok(())
}
