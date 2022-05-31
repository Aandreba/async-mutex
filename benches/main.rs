use std::sync::Arc;
use criterion::{Criterion, criterion_group, criterion_main};
use tokio::runtime::{Builder};

mod regular;
pub use regular::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("async_mutex", |b| {
        let runtime = Builder::new_multi_thread().build().unwrap();
        b.to_async(runtime).iter(move || {
            let mutex = Arc::new(async_mutex::Mutex::new(0u32));
            bench_crate(mutex)
        })
    });

    c.bench_function("futures", |b| {
        let runtime = Builder::new_current_thread().build().unwrap();
        b.to_async(runtime).iter(move || {
            let mutex = Arc::new(futures::lock::Mutex::new(0u32));
            bench_futures(mutex)
        })
    });

    c.bench_function("tokio", |b| {
        let runtime = Builder::new_current_thread().build().unwrap();
        b.to_async(runtime).iter(move || {
            let mutex = Arc::new(tokio::sync::Mutex::new(0u32));
            bench_tokio(mutex)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);