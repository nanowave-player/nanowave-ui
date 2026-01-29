use std::thread;
use std::time::Duration;

pub fn start_tokio_background_tasks() {
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(background_tasks());
    });
}

pub async fn background_tasks() {
    let countertask1 = tokio::spawn(async {
        let mut counter = 0;
        loop {
            counter = counter+1;
            println!("countertask 1: {}", counter);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    let countertask2 = tokio::spawn(async {
        let mut counter = 0;
        loop {
            counter = counter+1;
            println!("countertask 2: {}", counter);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    let _ = tokio::join!(countertask1, countertask2);
}