use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tombi_glob::MultiGlob;

fn benchmark_simple_pattern(c: &mut Criterion) {
    c.bench_function("simple pattern matching", |b| {
        let mut glob = MultiGlob::new();
        glob.add("hello", 1).unwrap();
        glob.compile();

        b.iter(|| {
            black_box(glob.find(black_box("hello")));
        });
    });
}

fn benchmark_wildcard_pattern(c: &mut Criterion) {
    c.bench_function("wildcard pattern matching", |b| {
        let mut glob = MultiGlob::new();
        glob.add("*.rs", 1).unwrap();
        glob.compile();

        b.iter(|| {
            black_box(glob.find(black_box("main.rs")));
        });
    });
}

fn benchmark_complex_pattern(c: &mut Criterion) {
    c.bench_function("complex pattern matching", |b| {
        let mut glob = MultiGlob::new();
        glob.add("src/**/*.{rs,toml}", 1).unwrap();
        glob.compile();

        b.iter(|| {
            black_box(glob.find(black_box("src/lib/main.rs")));
        });
    });
}

fn benchmark_multiple_patterns(c: &mut Criterion) {
    c.bench_function("multiple patterns matching", |b| {
        let mut glob = MultiGlob::new();
        glob.add("*.txt", 10).unwrap();
        glob.add("test.*", 5).unwrap();
        glob.add("test.txt", 20).unwrap();
        glob.add("**/*.rs", 15).unwrap();
        glob.add("src/**/*.toml", 12).unwrap();
        glob.compile();

        b.iter(|| {
            black_box(glob.find(black_box("test.txt")));
            black_box(glob.find(black_box("hello.txt")));
            black_box(glob.find(black_box("test.rs")));
            black_box(glob.find(black_box("src/cargo.toml")));
        });
    });
}

criterion_group!(
    benches,
    benchmark_simple_pattern,
    benchmark_wildcard_pattern,
    benchmark_complex_pattern,
    benchmark_multiple_patterns
);

criterion_main!(benches);
