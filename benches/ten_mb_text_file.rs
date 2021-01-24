use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("ten_mb text file", |b| {
        b.iter(|| {
            tc::count_file(
                &["test_data", "ten_mb.txt"].iter().collect(),
                &tc::Config {
                    bytes: true,
                    chars: true,
                    words: true,
                    tokens: true,
                    lines: true,
                },
            );
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
