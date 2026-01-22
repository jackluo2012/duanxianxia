//! Storage Service性能基准测试
//!
//! 测试批量写入和查询性能

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;
use storage_domain::{BatchConfig, DataBatch};

/// 批次写入性能测试
fn bench_batch_add(c: &mut Criterion) {
    let config = BatchConfig::default(); // 100条或5秒
    let mut batch = DataBatch::new(config);

    c.bench_function("batch_add_single", |b| {
        b.iter(|| {
            let item = serde_json::json!({
                "code": "000001",
                "price": 10.5,
                "volume": 1000
            });
            batch.add(item);
        });
    });

    // 测试批量添加
    c.bench_function("batch_add_10_items", |b| {
        b.iter(|| {
            let items: Vec<serde_json::Value> =
                (0..10).map(|i| serde_json::json!({"index": i})).collect();
            batch.add_batch(items);
        });
    });

    c.bench_function("batch_add_100_items", |b| {
        b.iter(|| {
            let items: Vec<serde_json::Value> =
                (0..100).map(|i| serde_json::json!({"index": i})).collect();
            let mut batch = DataBatch::new(config);
            batch.add_batch(items);
        });
    });
}

/// JSON序列化性能测试
fn bench_json_serialization(c: &mut Criterion) {
    let quote = serde_json::json!({
        "code": "000001",
        "name": "平安银行",
        "price": 10.5,
        "open": 10.2,
        "high": 10.8,
        "low": 10.1,
        "close": 10.5,
        "volume": 1000000,
        "datetime": "2026-01-15T09:30:00Z"
    });

    c.bench_function("json_serialize", |b| {
        b.iter(|| {
            serde_json::to_vec(&quote).unwrap();
        });
    });
}

criterion_group!(benches, bench_batch_add, bench_json_serialization);
criterion_main!(benches);
