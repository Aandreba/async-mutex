
use std::sync::Arc;
use futures::future::{try_join_all};

const SIZE : usize = 100;

pub async fn bench_crate (mutex: Arc<async_mutex::Mutex<u32>>) {
    let mut handles = Vec::with_capacity(SIZE);

    for _ in 0..SIZE {
        let mutex = mutex.clone();
        handles.push(tokio::spawn(async move {
            let mut mutex = mutex.lock().await;
            *mutex += 1;
        }));
    }
    
    try_join_all(handles).await.unwrap();
}

pub async fn bench_futures (mutex: Arc<futures::lock::Mutex<u32>>) {
    let mut handles = Vec::with_capacity(SIZE);

    for _ in 0..SIZE {
        let mutex = mutex.clone();
        handles.push(tokio::spawn(async move {
            let mut mutex = mutex.lock().await;
            *mutex += 1;
        }));
    }
    
    try_join_all(handles).await.unwrap();
}

pub async fn bench_tokio (mutex: Arc<tokio::sync::Mutex<u32>>) {
    let mut handles = Vec::with_capacity(SIZE);

    for _ in 0..SIZE {
        let mutex = mutex.clone();
        handles.push(tokio::spawn(async move {
            let mut mutex = mutex.lock().await;
            *mutex += 1;
        }));
    }
    
    try_join_all(handles).await.unwrap();
}