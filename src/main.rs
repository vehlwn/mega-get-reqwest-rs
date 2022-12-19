use std::sync::{Arc, Mutex};

#[derive(Default, Debug, Copy, Clone)]
struct ProgressBar {
    current_value: usize,
    total_value: usize,
}
impl std::fmt::Display for ProgressBar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return write!(f, "{}/{}", self.current_value, self.total_value);
    }
}

async fn perform_get_impl(
    client: &reqwest::Client,
    url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let req = client.get(url).build()?;
    log::trace!(
        "Request line: {} {} {:?}",
        req.method().as_str(),
        req.url().as_str(),
        req.version()
    );
    let res = client.execute(req).await?;
    log::debug!("Response status = {}", res.status());
    log::trace!("Response headers = {:?}", res.headers());
    if let Some(content_length) = res.content_length() {
        log::trace!("content_length = {}", content_length);
        log::trace!("Response body = {}", res.text().await?);
    }
    return Ok(());
}

fn get_default_headers() -> reqwest::header::HeaderMap {
    let mut ret = reqwest::header::HeaderMap::new();
    ret.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("*/*"),
    );
    ret.insert(
        reqwest::header::CONNECTION,
        reqwest::header::HeaderValue::from_static("keep-alive"),
    );
    ret.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:70.0) Gecko/20100101 Firefox/70.0",
        ),
    );
    return ret;
}

fn build_client(args: &Cli) -> Result<reqwest::Client, Box<dyn std::error::Error>> {
    let mut client_builder = reqwest::ClientBuilder::new().default_headers(get_default_headers());
    client_builder = match args.proxy {
        Some(ref p) => client_builder.proxy(reqwest::Proxy::all(p)?),
        None => client_builder,
    };
    return Ok(client_builder.build()?);
}

async fn main_coroutine(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = Vec::new();
    let rem = args.requests.get() % args.world.get();
    let quot = args.requests.get() / args.world.get();
    let progress = Arc::new(Mutex::new(ProgressBar {
        current_value: 0,
        total_value: args.requests.get(),
    }));
    for thread_id in 0..args.world.get() {
        let client = build_client(&args)?;
        let url = args.url.clone();
        let requests_per_thread = {
            if thread_id < rem {
                quot + 1
            } else {
                quot
            }
        };
        log::debug!(
            "thread_id = {}, requests_per_thread = {}",
            thread_id,
            requests_per_thread
        );
        let progress = progress.clone();
        tasks.push(tokio::spawn(async move {
            for _ in 0..requests_per_thread {
                if let Err(e) = perform_get_impl(&client, &url).await {
                    log::error!("{:?}", e);
                }
                let mut lock = progress.lock().unwrap();
                lock.current_value += 1;
                log::info!("Done {} requests", lock);
            }
        }));
    }
    for task in tasks {
        task.await?;
    }
    return Ok(());
}

/// Program to perform repeated GET requests in batches
#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Number of threads
    #[arg(short, long, default_value = "4")]
    world: std::num::NonZeroUsize,

    /// Total number of requests
    #[arg(short, long, default_value = "100")]
    requests: std::num::NonZeroUsize,

    /// HTTP URL
    url: String,

    /// Proxy URL, e.g. `socks5h://localhost:9050`. For available proxy schemes see `impl
    /// ProxyScheme` in https://docs.rs/reqwest/latest/src/reqwest/proxy.rs.html
    #[arg(short, long)]
    proxy: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    use clap::Parser;
    let args = Cli::parse();
    log::debug!("args = {:?}", args);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(args.world.get())
        .enable_all()
        .build()?;

    return runtime.block_on(main_coroutine(args));
}
