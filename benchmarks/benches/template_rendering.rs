//! Benchmarks for template rendering via `TemplateEngine`.
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

fn bench_engine_creation(c: &mut Criterion) {
    c.bench_function("template_engine_new", |b| {
        b.iter(|| {
            let engine = nettoolskit_templating::TemplateEngine::new();
            black_box(engine);
        });
    });
}

fn bench_render_from_string_simple(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let engine = nettoolskit_templating::TemplateEngine::new();
    let template = "Hello, {{name}}!";
    let data = json!({"name": "World"});

    c.bench_function("render_from_string_simple", |b| {
        b.iter(|| {
            rt.block_on(async {
                engine
                    .render_from_string(
                        black_box(template),
                        black_box(&data),
                        "bench_simple".to_string(),
                    )
                    .await
                    .expect("render failed")
            })
        });
    });
}

fn bench_render_from_string_complex(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let engine = nettoolskit_templating::TemplateEngine::new();

    let template = r"using System;
namespace {{namespace}}.{{context}}.{{kind}};

/// <summary>
/// {{description}}
/// </summary>
public sealed class {{className}}
{
{{#each fields}}
    /// <summary>
    /// Gets or sets {{this.name}}.
    /// </summary>
    public {{this.type}} {{this.name}} { get; set; }{{#unless @last}}

{{/unless}}
{{/each}}
}";

    let data = json!({
        "namespace": "MyApp.Domain",
        "context": "Orders",
        "kind": "Entities",
        "description": "Represents an order in the system",
        "className": "Order",
        "fields": [
            {"name": "Id", "type": "Guid"},
            {"name": "CustomerId", "type": "Guid"},
            {"name": "CreatedAt", "type": "DateTime"},
            {"name": "TotalAmount", "type": "decimal"},
            {"name": "Status", "type": "OrderStatus"},
        ]
    });

    c.bench_function("render_from_string_complex", |b| {
        b.iter(|| {
            rt.block_on(async {
                engine
                    .render_from_string(
                        black_box(template),
                        black_box(&data),
                        "bench_complex".to_string(),
                    )
                    .await
                    .expect("render failed")
            })
        });
    });
}

fn bench_render_from_string_cached(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let engine = nettoolskit_templating::TemplateEngine::new();
    let template = "{{greeting}}, {{name}}! Welcome to {{project}}.";
    let data = json!({"greeting": "Hello", "name": "Developer", "project": "NetToolsKit"});

    // Warm up the cache with one render
    rt.block_on(async {
        engine
            .render_from_string(template, &data, "bench_cached".to_string())
            .await
            .expect("warmup render");
    });

    c.bench_function("render_from_string_cached", |b| {
        b.iter(|| {
            rt.block_on(async {
                engine
                    .render_from_string(
                        black_box(template),
                        black_box(&data),
                        "bench_cached".to_string(),
                    )
                    .await
                    .expect("render failed")
            })
        });
    });
}

criterion_group!(
    benches,
    bench_engine_creation,
    bench_render_from_string_simple,
    bench_render_from_string_complex,
    bench_render_from_string_cached,
);
criterion_main!(benches);
