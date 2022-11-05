use color_eyre::eyre;
use testcontainers_rs::*;

struct NginxContainer {
    container: Container,
}

impl NginxContainer {
    pub async fn start() -> eyre::Result<Self> {
        let container = Container {};
        // req := testcontainers.ContainerRequest{
        // Image:        "nginx",
        // ExposedPorts: []string{"80/tcp"},
        // WaitingFor:   wait.ForHTTP("/"),
        // }

        Ok(Self { container })
    }

    pub async fn uri(&self) -> eyre::Result<String> {
        // ip, err := container.Host(ctx)
        // if err != nil {
        // return nil, err
        // }

        // mappedPort, err := container.MappedPort(ctx, "80")
        // if err != nil {
        // return nil, err
        // }

        // uri := fmt.Sprintf("http://%s:%s", ip, mappedPort.Port())
        Ok("todo".to_string())
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let nginx = NginxContainer::start().await?;

    let resp = reqwest::get(nginx.uri().await?).await?.text().await?;
    println!("{:#?}", resp);
    // resp, err := http.Get(nginxC.URI)
    // if resp.StatusCode != http.StatusOK {
    // t.Fatalf("Expected status code %d. Got %d.", http.StatusOK, resp.StatusCode)
    // }

    // when the `NginxContainer` is dropped, the container is terminated
    Ok(())
}
