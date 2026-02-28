//! Benchmarks for UI formatting and string utility functions.
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nettoolskit_core::string_utils::string::{truncate_directory, truncate_directory_with_middle};
use nettoolskit_ui::core::formatting::format_menu_item as format_aligned;
use nettoolskit_ui::format_menu_item;

fn bench_format_menu_item_aligned(c: &mut Criterion) {
    c.bench_function("format_menu_item_aligned", |b| {
        b.iter(|| {
            format_aligned(
                black_box("/manifest"),
                black_box("Manage project manifests and scaffolding"),
                black_box(20),
            )
        });
    });
}

fn bench_format_menu_item_helpers(c: &mut Criterion) {
    c.bench_function("format_menu_item_helpers", |b| {
        b.iter(|| {
            format_menu_item(
                black_box("/manifest"),
                black_box(Some("Manage project manifests")),
            )
        });
    });
}

fn bench_format_menu_item_no_desc(c: &mut Criterion) {
    c.bench_function("format_menu_item_no_description", |b| {
        b.iter(|| format_menu_item(black_box("/manifest"), black_box(None)));
    });
}

fn bench_truncate_directory_short(c: &mut Criterion) {
    c.bench_function("truncate_directory_short", |b| {
        b.iter(|| truncate_directory(black_box("src/core/models"), black_box(40)));
    });
}

fn bench_truncate_directory_long(c: &mut Criterion) {
    let long_path = "src/domain/aggregates/orders/entities/order_items/value_objects";

    c.bench_function("truncate_directory_long", |b| {
        b.iter(|| truncate_directory(black_box(long_path), black_box(30)));
    });
}

fn bench_truncate_directory_with_middle(c: &mut Criterion) {
    let long_path = "src/domain/aggregates/orders/entities/order_items/value_objects";

    c.bench_function("truncate_directory_with_middle", |b| {
        b.iter(|| truncate_directory_with_middle(black_box(long_path), black_box(30)));
    });
}

criterion_group!(
    benches,
    bench_format_menu_item_aligned,
    bench_format_menu_item_helpers,
    bench_format_menu_item_no_desc,
    bench_truncate_directory_short,
    bench_truncate_directory_long,
    bench_truncate_directory_with_middle,
);
criterion_main!(benches);
