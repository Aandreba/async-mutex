use std::{sync::Arc, thread};
use async_mutex::movable::MovableMutex;
use futures::future::join_all;

#[test]
fn only_sync () {
    let data = Box::leak(Box::new(0u32));
    let mutex = Arc::new((MovableMutex::new(), data));
    let mut handles = Vec::with_capacity(100);

    for i in 0..8 {
        let mutex = mutex.clone();
        handles.push(thread::spawn(move || {
            for j in 0..10 {
                mutex.0.lock_blocking();
                unsafe {
                    let data = &mut *(mutex.1 as *const u32 as *mut u32);
                    *data += 1;
                    println!("({i}, {j}) = {data}");
                    mutex.0.unlock();
                };
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*mutex.1, 80);
}

#[tokio::test(flavor = "multi_thread")]
async fn only_async () {
    let data = Box::leak(Box::new(0u32));
    let mutex = Arc::new((MovableMutex::new(), data));
    let mut handles = Vec::with_capacity(1000);

    for _ in 0..1000 {
        let mutex = mutex.clone();
        handles.push(tokio::spawn(async move {
            mutex.0.lock().await;
            unsafe {
                let data = &mut *(mutex.1 as *const u32 as *mut u32);
                *data += 1;
                mutex.0.unlock();
            };
        }));
    }

    join_all(handles).await;
    assert_eq!(*mutex.1, 1000);
}

#[tokio::test(flavor = "multi_thread")]
async fn mixed () {
    let data = Box::leak(Box::new(0u32));
    let mutex = Arc::new((MovableMutex::new(), data));
    let mut handles = Vec::with_capacity(1000);

    for _ in 0..1000 {
        let mutex = mutex.clone();
        
        handles.push(tokio::spawn(async move {
            if rand::random::<bool>() {
                mutex.0.lock_blocking();
            } else {
                mutex.0.lock().await;
            }

            unsafe {
                let data = &mut *(mutex.1 as *const u32 as *mut u32);
                *data += 1;
                mutex.0.unlock();
            };
        }));
    }

    join_all(handles).await;
    assert_eq!(*mutex.1, 1000);
}