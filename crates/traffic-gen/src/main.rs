use anyhow::Context;
use chrono::Utc;
use clap::Parser;
use num_format::{Locale, ToFormattedString};
use rfc5321::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;
use uuid::Uuid;

const DOMAINS: &[&str] = &["aol.com", "gmail.com", "hotmail.com", "yahoo.com"];

#[derive(Clone, Debug, Parser)]
#[command(about = "SMTP traffic generator")]
struct Opt {
    /// All generated mail will have this domain appended.
    /// The default is an MX that routes to a loopback address.
    #[arg(long, default_value = "mx-sink.wezfurlong.org")]
    domain: String,

    /// The target host to which mail will be submitted
    #[arg(long, default_value = "127.0.0.1:2025")]
    target: String,

    /// The number of connections to open to target
    #[arg(long)]
    concurrency: Option<usize>,

    /// How many seconds to generate for
    #[arg(long, default_value = "60")]
    duration: u64,

    /// Whether to use STARTTLS for submission
    #[arg(long)]
    starttls: bool,
}

impl Opt {
    fn generate_sender(&self) -> String {
        format!("noreply@{}", self.domain)
    }

    fn generate_recipient(&self) -> String {
        let number: usize = rand::random();
        let domain = DOMAINS[number % DOMAINS.len()];
        format!("user-{number}@{domain}.{}", self.domain)
    }

    fn generate_body(&self, sender: &str, recip: &str) -> String {
        let now = Utc::now();
        let datestamp = now.to_rfc2822();
        let id = Uuid::new_v4().simple().to_string();

        format!(
            "From: <{sender}>\r\n\
             To: <{recip}>\r\n\
             Subject: test {datestamp}\r\n\
             Message-Id: {id}\r\n\
             X-Mailer: KumoMta traffic-gen\r\n\
             \r\n\
             This is a test message.\r\n\
             Enjoy!\r\n"
        )
    }

    fn generate_message(&self) -> (ReversePath, ForwardPath, String) {
        let sender = self.generate_sender();
        let recip = self.generate_recipient();
        let body = self.generate_body(&sender, &recip);
        (
            ReversePath::try_from(sender.as_str()).unwrap(),
            ForwardPath::try_from(recip.as_str()).unwrap(),
            body,
        )
    }

    async fn make_client(&self) -> anyhow::Result<SmtpClient> {
        let stream = TcpStream::connect(&self.target)
            .await
            .with_context(|| format!("connect to {}", self.target))?;
        let mut client = SmtpClient::with_stream(stream, &self.target);

        // Read banner
        let banner = client.read_response().await?;
        anyhow::ensure!(banner.code == 220, "unexpected banner: {banner:#?}");

        // Say EHLO
        let caps = client.ehlo(&self.domain).await?;

        if self.starttls && caps.contains_key("STARTTLS") {
            client.starttls(true).await?;
        }

        Ok(client)
    }

    async fn run(&self, counter: Arc<AtomicUsize>) -> anyhow::Result<()> {
        let mut client = self.make_client().await?;
        let started = Instant::now();
        let duration = Duration::from_secs(self.duration);
        while started.elapsed() <= duration {
            let (sender, recip, body) = self.generate_message();
            timeout(
                Duration::from_secs(300),
                client.send_mail(sender, recip, body),
            )
            .await
            .context("waiting to send mail")?
            .context("sending mail")?;
            counter.fetch_add(1, Ordering::Relaxed);
        }
        timeout(Duration::from_secs(1), client.send_command(&Command::Quit)).await??;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opt::parse();

    let counter = Arc::new(AtomicUsize::new(0));
    let started = Instant::now();
    let duration = Duration::from_secs(opts.duration);
    let concurrency = match opts.concurrency {
        Some(n) => n,
        None => {
            let n_threads: usize = std::thread::available_parallelism()?.into();
            n_threads * 10
        }
    };

    for _ in 0..concurrency {
        let opts = opts.clone();
        let counter = Arc::clone(&counter);
        tokio::spawn(async move {
            if let Err(err) = opts.run(counter).await {
                eprintln!("{err:#}");
            }
            Ok::<(), anyhow::Error>(())
        });
    }

    tokio::time::sleep(duration).await;
    let total_sent = counter.load(Ordering::Acquire);
    let elapsed = started.elapsed();

    let msgs_per_second = total_sent as f64 / elapsed.as_secs_f64();

    let msgs_per_minute = msgs_per_second * 60.;
    let msgs_per_hour = msgs_per_minute * 60.;

    let msgs_per_minute = (msgs_per_minute as usize).to_formatted_string(&Locale::en);
    let msgs_per_hour = (msgs_per_hour as usize).to_formatted_string(&Locale::en);

    println!("did {total_sent} messages over {elapsed:?}.");
    println!("{msgs_per_second} msgs/s, {msgs_per_minute} msgs/minute, {msgs_per_hour} msgs/hour");

    Ok(())
}