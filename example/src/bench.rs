use scpi::prelude::*;

mod common;
use common::*;

use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut my_device = MyDevice {};
    let mut errors = ArrayErrorQueue::<[Error; 10]>::new();
    let mut context = Context::new(&mut my_device, &mut errors, TREE);
    //Response bytebuffer
    let mut buf = ArrayVecFormatter::<[u8; 256]>::new();
    c.bench_function("scpi", |b| {
        b.iter(|| {
            //Result
            let result = context.run(b"syst:version?", &mut buf);
            assert!(result.is_ok())
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
