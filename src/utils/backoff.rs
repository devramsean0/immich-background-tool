use tokio::time::{sleep, Duration};

pub async fn backoff_delay(mut delay: &u64, counter: &i64) {
    let delay_amt = &((counter * 2000) as u64).max(1);
    delay = delay_amt;
    println!("Sleeping for {}", delay);
    sleep(Duration::from_millis(*delay)).await;
}
