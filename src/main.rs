#[derive(Debug, clap::Parser)]
struct Args {
    /// Bind address
    #[clap(short, long, default_value = "127.0.0.1:8080")]
    bind: String,
    /// Upstream root URL
    #[clap(short, long)]
    upstream_url: url::Url,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use clap::Parser as _;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    let args = Args::parse();
    let upstream_url = std::sync::Arc::new(args.upstream_url);

    let make_service = hyper::service::make_service_fn(move |_| {
        let u = upstream_url.clone();
        async {
            Ok::<_, std::convert::Infallible>(hyper::service::service_fn(move |r| {
                handle(u.clone(), r)
            }))
        }
    });
    let server = if let Some(listener) = listenfd::ListenFd::from_env().take_tcp_listener(0)? {
        tracing::info!("Listen {}", listener.local_addr()?);
        hyper::server::Server::from_tcp(listener)?
    } else {
        let addr = args.bind.parse()?;
        tracing::info!("Listen {}", addr);
        hyper::server::Server::bind(&addr)
    }
    .serve(make_service)
    .with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("Shutting down...");
    });
    server.await?;
    Ok(())
}

async fn handle(
    upstream_url: std::sync::Arc<url::Url>,
    mut request: hyper::Request<hyper::Body>,
) -> anyhow::Result<hyper::Response<hyper::Body>> {
    tracing::info!(method = %request.method(), path = %request.uri().path(), "Received request from downstream");
    for (key, value) in request.headers() {
        println!("{}: {}", key, value.to_str().unwrap_or("(ToStrError)"));
    }
    println!();

    let mut u = upstream_url.join(request.uri().path().trim_start_matches('/'))?;
    u.set_query(request.uri().query());
    use std::str::FromStr as _;
    *request.uri_mut() = hyper::Uri::from_str(u.as_str())?;

    request.headers_mut().remove(http::header::HOST);

    let request = request.map(|body| hyper::Body::wrap_stream(with_body_logging(body)));

    if request.uri().scheme() == Some(&http::uri::Scheme::HTTPS) {
        proxy(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_only()
                    .enable_http1()
                    .build(),
            ),
            request,
        )
        .await
    } else {
        proxy(hyper::Client::new(), request).await
    }
}

async fn proxy<C>(
    client: hyper::Client<C>,
    request: hyper::Request<hyper::Body>,
) -> anyhow::Result<hyper::Response<hyper::Body>>
where
    C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    tracing::info!(uri = %request.uri(), "Send request to upstream");
    let resp = client.request(request).await?;

    tracing::info!(status = %resp.status(), "Received response from upstream");
    for (key, value) in resp.headers() {
        println!("{}: {}", key, value.to_str().unwrap_or("(ToStrError)"));
    }
    println!();

    let resp = resp.map(|body| hyper::Body::wrap_stream(with_body_logging(body)));

    Ok(resp)
}

fn with_body_logging(
    body: hyper::Body,
) -> impl futures::Stream<Item = Result<hyper::body::Bytes, hyper::Error>> {
    use futures::TryStreamExt as _;
    body.map_ok(|body| {
        use std::io::Write as _;
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(&body).unwrap();
        stdout.flush().unwrap();
        body
    })
}
