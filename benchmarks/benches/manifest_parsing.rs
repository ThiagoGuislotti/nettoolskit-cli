//! Benchmarks for manifest YAML parsing and validation.
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nettoolskit_manifest::models::ManifestDocument;

const MINIMAL_MANIFEST: &str = r#"apiVersion: ntk/v1
kind: solution
meta:
  name: TestManifest
conventions:
  namespaceRoot: TestApp
  targetFramework: net9.0
solution:
  root: ./
  slnFile: TestApp.sln
apply:
  mode: artifact
  artifact:
    kind: entity
    project: Domain
    namespace: Core.Entities
    context: Orders
"#;

const RICH_MANIFEST: &str = r#"apiVersion: ntk/v1
kind: solution
meta:
  name: test-manifest
solution:
  root: ./
  slnFile: TestSolution.sln
conventions:
  namespaceRoot: MyApp
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: true
    strict: false
apply:
  mode: feature
  feature:
    context: Orders
    include: []
contexts:
  - name: Orders
    aggregates:
      - name: Order
        valueObjects:
          - name: OrderId
            fields:
              - name: value
                type: Guid
                nullable: false
        entities:
          - name: OrderItem
            fields:
              - name: quantity
                type: int
                nullable: false
                key: false
                columnName: null
            isRoot: false
    useCases:
      - name: CreateOrder
        type: command
        input:
          - name: customerId
            type: Guid
            nullable: false
            key: false
            columnName: null
        output:
          - name: orderId
            type: Guid
            nullable: false
            key: false
            columnName: null
  - name: Catalog
    aggregates:
      - name: Product
        valueObjects:
          - name: ProductId
            fields:
              - name: value
                type: Guid
                nullable: false
          - name: Money
            fields:
              - name: amount
                type: decimal
                nullable: false
              - name: currency
                type: string
                nullable: false
        entities:
          - name: ProductVariant
            fields:
              - name: sku
                type: string
                nullable: false
                key: true
                columnName: null
              - name: price
                type: decimal
                nullable: false
                key: false
                columnName: null
            isRoot: false
    useCases:
      - name: CreateProduct
        type: command
        input:
          - name: name
            type: string
            nullable: false
            key: false
            columnName: null
          - name: price
            type: decimal
            nullable: false
            key: false
            columnName: null
        output:
          - name: productId
            type: Guid
            nullable: false
            key: false
            columnName: null
      - name: GetProduct
        type: query
        input:
          - name: productId
            type: Guid
            nullable: false
            key: false
            columnName: null
        output:
          - name: product
            type: Product
            nullable: true
            key: false
            columnName: null
"#;

fn bench_parse_minimal_manifest(c: &mut Criterion) {
    c.bench_function("parse_manifest_minimal", |b| {
        b.iter(|| {
            serde_yaml::from_str::<ManifestDocument>(black_box(MINIMAL_MANIFEST))
                .expect("parse failed")
        });
    });
}

fn bench_parse_rich_manifest(c: &mut Criterion) {
    c.bench_function("parse_manifest_rich", |b| {
        b.iter(|| {
            serde_yaml::from_str::<ManifestDocument>(black_box(RICH_MANIFEST))
                .expect("parse failed")
        });
    });
}

fn bench_validate_manifest(c: &mut Criterion) {
    let doc: ManifestDocument =
        serde_yaml::from_str(RICH_MANIFEST).expect("parse for validation setup");

    c.bench_function("validate_manifest", |b| {
        b.iter(|| {
            nettoolskit_manifest::ManifestParser::validate(black_box(&doc))
                .expect("validate failed")
        });
    });
}

criterion_group!(
    benches,
    bench_parse_minimal_manifest,
    bench_parse_rich_manifest,
    bench_validate_manifest,
);
criterion_main!(benches);
