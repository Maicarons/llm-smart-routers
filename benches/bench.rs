//! LLM Smart Router 性能基准测试
//!
//! 运行方式: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_protocol_conversion(c: &mut Criterion) {
    let openai_body = br#"{"model":"gpt-4o","messages":[{"role":"user","content":"Hello world"}],"max_tokens":100}"#;
    let openai_converter = llm_smart_router_protocol::converter::OpenAI;

    c.bench_function("openai_to_unified", |b| {
        b.iter(|| {
            let _ = openai_converter.to_unified_request(black_box(openai_body));
        });
    });
}

fn bench_task_classification(c: &mut Criterion) {
    let classifier = llm_smart_router_router_engine::classifier::Classifier::new();
    c.bench_function("classify_task", |b| {
        b.iter(|| {
            let _ = classifier.classify(black_box("Write a Python function to sort an array"));
        });
    });
}

fn bench_circuit_breaker(c: &mut Criterion) {
    let config = llm_smart_router_circuit_breaker::config::Config::default();
    let cb = llm_smart_router_circuit_breaker::CircuitBreaker::new(config);
    c.bench_function("breaker_record_check", |b| {
        b.iter(|| {
            cb.record("test-model", true, 100, None);
            let _ = cb.check("test-model");
        });
    });
}

fn bench_scorer(c: &mut Criterion) {
    let scorer = llm_smart_router_router_engine::scorer::Scorer::default();
    let profile = llm_smart_router_router_engine::scorer::ModelCapabilityProfile::estimate("gpt-4o", "openai");
    c.bench_function("scorer_score", |b| {
        b.iter(|| {
            let _ = scorer.score(black_box(&profile), None);
        });
    });
}

criterion_group!(benches, bench_protocol_conversion, bench_task_classification, bench_circuit_breaker, bench_scorer);
criterion_main!(benches);