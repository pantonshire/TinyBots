use std::env;

use anyhow::Context;
use chrono::Utc;
use rand::Rng;

const VAR_GOLDCREST_SCHEME: &str = "GOLDCREST_SCHEME";
const VAR_GOLDCREST_HOST: &str = "GOLDCREST_HOST";
const VAR_GOLDCREST_PORT: &str = "GOLDCREST_PORT";
const VAR_GOLDCREST_REQUEST_TIMEOUT: &str = "GOLDCREST_REQUEST_TIMEOUT";
const VAR_GOLDCREST_WAIT_TIMEOUT: &str = "GOLDCREST_WAIT_TIMEOUT";

const VAR_TWITTER_CONSUMER_KEY: &str = "TWITTER_CONSUMER_KEY";
const VAR_TWITTER_CONSUMER_SECRET: &str = "TWITTER_CONSUMER_SECRET";
const VAR_TWITTER_TOKEN: &str = "TWITTER_TOKEN";
const VAR_TWITTER_TOKEN_SECRET: &str = "TWITTER_TOKEN_SECRET";

const TRANS_FLAG: &str = "\u{1f3f3}\u{0fe0f}\u{0200d}\u{026a7}\u{0fe0f}";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "dotenv")] {
        dotenv::dotenv().ok();
    }

    let client = connect_goldcrest().await?;

    let weighted_tweet_ids: Vec<(u64, u32)> = vec![
        // Transrightsbot
        (1270973597885050880, 3),
        // Nonbinarybot
        (1103565026571489281, 1),
        // Transbot
        (1162253080110354433, 1),
        // Outbot
        (940115273864110080,  1),
        // Genderbot
        (934112336960544768,  1),
        // Transwrongsbot
        (1615276641013448704, 3),
    ];

    let total_w = weighted_tweet_ids
        .iter()
        .map(|(_, w)| w)
        .sum::<u32>();

    let cumulative_tweet_ids: Vec<(u64, u32)> = weighted_tweet_ids
        .into_iter()
        .scan(0, |acc, (id, w)| {
            *acc += w;
            Some((id, *acc))
        })
        .collect();

    let rand_w = rand::thread_rng().gen_range(0..total_w);

    let tweet_id = cumulative_tweet_ids
        .into_iter()
        .find(|(_, w)| rand_w < *w)
        .unwrap().0;

    let now = Utc::now()
        .format("%d/%m/%y")
        .to_string();

    let text = format!("{} {}{}{} https://twitter.com/smolrobots/status/{}",
        now, TRANS_FLAG, TRANS_FLAG, TRANS_FLAG, tweet_id);

    let builder = goldcrest::request::TweetBuilder::new(text);

    client.publish(builder, goldcrest::request::TweetOptions::default())
        .await
        .context("failed to send tweet")?;

    Ok(())
}

async fn connect_goldcrest() -> anyhow::Result<goldcrest::Client> {
    let consumer_key = env::var(VAR_TWITTER_CONSUMER_KEY)
        .context("twitter consumer key missing or invalid")?;
    let consumer_secret = env::var(VAR_TWITTER_CONSUMER_SECRET)
        .context("twitter consumer secret missing or invalid")?;
    let token = env::var(VAR_TWITTER_TOKEN)
        .context("twitter token missing or invalid")?;
    let token_secret = env::var(VAR_TWITTER_TOKEN_SECRET)
        .context("twitter token secret missing or invalid")?;

    let auth = goldcrest::Authentication::new(
        consumer_key,
        consumer_secret,
        token,
        token_secret
    );

    let mut client = goldcrest::ClientBuilder::new();
    client.authenticate(auth);

    if let Some(scheme) = opt_env_var(VAR_GOLDCREST_SCHEME)
        .context("error reading goldcrest scheme")?
    {
        client.scheme(&scheme);
    }

    if let Some(host) = opt_env_var(VAR_GOLDCREST_HOST)
        .context("error reading goldcrest host")?
    {
        client.host(&host);
    }

    if let Some(port) = opt_env_var(VAR_GOLDCREST_PORT)
        .context("error reading goldcrest port")?
    {
        let port = port.parse::<u16>()
            .with_context(|| format!(r#"invalid port "{}""#, port))?;
        client.port(port as u32);
    }

    if let Some(timeout) = opt_env_var(VAR_GOLDCREST_REQUEST_TIMEOUT)
        .context("error reading goldcrest request timeout")?
    {
        let timeout = timeout.parse::<i64>()
            .with_context(|| format!(r#"invalid request timeout "{}""#, timeout))?;
        client.request_timeout(chrono::Duration::seconds(timeout));
    }

    if let Some(timeout) = opt_env_var(VAR_GOLDCREST_WAIT_TIMEOUT)
        .context("error reading goldcrest request timeout")?
    {
        let timeout = timeout.parse::<i64>()
            .with_context(|| format!(r#"invalid wait timeout "{}""#, timeout))?;
        client.wait_timeout(chrono::Duration::seconds(timeout));
    }

    client.connect()
        .await
        .context("failed to connect to goldcrest")
}

fn opt_env_var(key: &str) -> Result<Option<String>, env::VarError> {
    match env::var(key) {
        Ok(val) => Ok(Some(val)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err @ env::VarError::NotUnicode(_)) => Err(err),
    }
}
