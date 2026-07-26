//! Benchmarks for parsing OVER documents.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use over::obj::Obj;
use std::str::FromStr;

const EXAMPLE: &str = include_str!("../tests/test_files/example.over");

// A flat document with many integer fields.
fn gen_ints(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&format!("field_{}: {}\n", i, i * 7));
    }
    s
}

// A flat document with many string fields.
fn gen_strs(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&format!(
            "field_{}: \"The quick brown fox jumps over the lazy dog {}\"\n",
            i, i
        ));
    }
    s
}

// A document with many nested objects containing arrays and mixed values.
fn gen_nested(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&format!(
            "item_{}: {{\n    name: \"item {}\"\n    price: {}.{:02}\n    counts: [1 2 3 4 5]\n    tags: (\"a\" {} true)\n}}\n",
            i,
            i,
            i,
            i % 100,
            i
        ));
    }
    s
}

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    let inputs = [
        ("example", EXAMPLE.to_string()),
        ("ints_1000", gen_ints(1000)),
        ("strs_500", gen_strs(500)),
        ("nested_200", gen_nested(200)),
    ];

    for (name, input) in &inputs {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_function(*name, |b| {
            b.iter(|| Obj::from_str(black_box(input)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
