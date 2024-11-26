use criterion::{criterion_group, criterion_main, Criterion};
use custom_nosql_cdn::database::Database;
use std::fs::remove_file;

fn bench_database_insert(c: &mut Criterion) {
    // Clean up any previous benchmark data
    let _ = remove_file("bench_data.db");
    let db = Database::new("bench_data.db".to_string());

    c.bench_function("database_insert", |b| {
        b.iter(|| {
            db.insert("key", b"value").unwrap();
        });
    });
}

fn bench_database_get(c: &mut Criterion) {
    let db = Database::new("bench_data.db".to_string());
    db.insert("key", b"value").unwrap();

    c.bench_function("database_get", |b| {
        b.iter(|| {
            db.get("key").unwrap();
        });
    });
}

criterion_group!(benches, bench_database_insert, bench_database_get);
criterion_main!(benches);
