use std::{sync::Arc, thread};
use async_mutex::Mutex;
use futures::future::{join_all, try_join_all};

const SIZE : usize = 10_000;

#[test]
fn only_sync () {
    let mutex = Arc::new(Mutex::new(0));
    let mut handles = Vec::with_capacity(SIZE);

    for _ in 0..8 {
        let mutex = mutex.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..(SIZE/8) {
                let mut data = mutex.lock_blocking();
                *data += 1;
                println!("{}", *data)
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let inner = Arc::try_unwrap(mutex).unwrap();
    assert_eq!(inner.into_inner(), SIZE);
}

#[tokio::test(flavor = "multi_thread")]
async fn only_async () {
    let mutex = Arc::new(Mutex::new(0));
    let mut handles = Vec::with_capacity(SIZE);

    for _ in 0..SIZE {
        let mutex = mutex.clone();
        handles.push(tokio::spawn(async move {
            let mut data = mutex.lock().await;
            *data += 1;
            println!("{}", *data)
        }));
    }

    try_join_all(handles).await.unwrap();
    let inner = Arc::try_unwrap(mutex).unwrap();
    assert_eq!(inner.into_inner(), SIZE);
}

#[tokio::test(flavor = "multi_thread")]
async fn mixed () {
    let mutex = Arc::new(Mutex::new(0));
    let mut handles = Vec::with_capacity(SIZE);

    for _ in 0..SIZE {
        let mutex = mutex.clone();
        handles.push(tokio::spawn(async move {
            let mut data = if rand::random::<bool>() {
                mutex.lock_blocking()
            } else {
                mutex.lock().await
            };

            *data += 1;
            println!("{}", *data)
        }));
    }

    join_all(handles).await;
    let inner = Arc::try_unwrap(mutex).unwrap();
    assert_eq!(inner.into_inner(), SIZE);
}