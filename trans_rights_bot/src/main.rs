use rand::Rng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth_file = include_str!("../auth");

    let mut auth_file = auth_file
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());

    let consumer_key = auth_file
        .next()
        .expect("auth file does not contain a consumer key");
    let consumer_secret = auth_file
        .next()
        .expect("auth file does not contain a consumer secret");
    let token = auth_file
        .next()
        .expect("auth file does not contain a token");
    let token_secret = auth_file
        .next()
        .expect("auth file does not contain a token secret");

    let auth = goldcrest::Authentication::new(consumer_key, consumer_secret, token, token_secret);

    let mut client = goldcrest::ClientBuilder::new();
    client
        .authenticate(auth)
        .port(7400)
        .request_timeout(chrono::Duration::seconds(30))
        .wait_timeout(chrono::Duration::minutes(60));

    let client = client.connect().await?;

    let tweet_opts = goldcrest::request::TweetOptions::default();

    let weighted_tweet_ids: Vec<(u64, u32)> = vec![
        (1270973597885050880, 10), //Transrightsbot
        (1103565026571489281, 1),  //Nonbinarybot
        (1162253080110354433, 1),  //Transbot
    ];

    let total_w = weighted_tweet_ids
        .iter()
        .map(|(_,w)| w)
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

    let tweet = client.get_tweet(tweet_id, tweet_opts.clone()).await?;

    if tweet.you_retweeted {
        client.unretweet(tweet_id, tweet_opts.clone()).await?;
    }

    client.retweet(tweet_id, tweet_opts.clone()).await?;

    Ok(())
}
