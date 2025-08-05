//! Performance benchmarks for Casbin authorization system
//! Tests throughput, latency, and cache performance under load

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use epsx::dom::services::casbin_service::CasbinService;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark setup helper
struct BenchmarkContext {
    casbin_service: Arc<CasbinService>,
    rt: Runtime,
}

impl BenchmarkContext {
    fn new() -> Self {
        let rt = Runtime::new().unwrap();
        
        let casbin_service = rt.block_on(async {
            let database_url = std::env::var("BENCHMARK_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_benchmark_db".to_string());
            
            let pool = PgPool::connect(&database_url)
                .await
                .expect("Failed to connect to benchmark database");
            
            // Run migrations
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("Failed to run migrations");
            
            // Setup test policies
            let service = Arc::new(CasbinService::new(pool).await.expect("Failed to create CasbinService"));
            
            // Add role hierarchy for benchmarking
            service.add_role_for_user("admin_user", "admin").await.unwrap();
            service.add_role_for_user("moderator_user", "moderator").await.unwrap(); 
            service.add_role_for_user("premium_user", "premium_user").await.unwrap();
            service.add_role_for_user("basic_user", "basic_user").await.unwrap();
            
            // Add policies for different resources
            for i in 0..100 {
                service.add_policy("admin", &format!("/api/v1/resource{}", i), "GET").await.unwrap();
                service.add_policy("moderator", &format!("/api/v1/resource{}", i), "GET").await.unwrap();
                service.add_policy("premium_user", &format!("/api/v1/premium{}", i), "GET").await.unwrap();
                service.add_policy("basic_user", &format!("/api/v1/basic{}", i), "GET").await.unwrap();
            }
            
            service
        });
        
        Self { casbin_service, rt }
    }
}

/// Benchmark single policy enforcement
fn bench_single_enforcement(c: &mut Criterion) {
    let ctx = BenchmarkContext::new();
    
    let mut group = c.benchmark_group("single_enforcement");
    group.throughput(Throughput::Elements(1));
    
    // Benchmark cold cache (first enforcement)
    group.bench_function("cold_cache", |b| {
        b.to_async(&ctx.rt).iter(|| async {
            // Clear cache before each iteration
            ctx.casbin_service.clear_cache().await;
            
            black_box(
                ctx.casbin_service
                    .enforce("admin_user", "/api/v1/resource1", "GET")
                    .await
                    .unwrap()
            )
        })
    });
    
    // Benchmark warm cache (repeated enforcement)
    group.bench_function("warm_cache", |b| {
        b.to_async(&ctx.rt).iter(|| async {
            black_box(
                ctx.casbin_service
                    .enforce("admin_user", "/api/v1/resource1", "GET")
                    .await
                    .unwrap()
            )
        })
    });
    
    group.finish();
}

/// Benchmark enforcement with different role complexities
fn bench_role_complexity(c: &mut Criterion) {
    let ctx = BenchmarkContext::new();
    
    let mut group = c.benchmark_group("role_complexity");
    
    let roles = vec![
        ("basic_user", "Basic user with limited permissions"),
        ("premium_user", "Premium user with extended permissions"),
        ("moderator_user", "Moderator with user management permissions"),
        ("admin_user", "Admin with full system access"),
    ];
    
    for (role, description) in roles {
        group.bench_with_input(
            BenchmarkId::new("enforce_by_role", description),
            &role,
            |b, &role| {
                b.to_async(&ctx.rt).iter(|| async {
                    black_box(
                        ctx.casbin_service
                            .enforce(role, "/api/v1/resource1", "GET")
                            .await
                            .unwrap()
                    )
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark concurrent enforcement
fn bench_concurrent_enforcement(c: &mut Criterion) {
    let ctx = BenchmarkContext::new();
    
    let mut group = c.benchmark_group("concurrent_enforcement");
    
    let concurrency_levels = vec![1, 10, 50, 100, 500];
    
    for &concurrency in &concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent_requests", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.to_async(&ctx.rt).iter(|| async {
                    let mut handles = Vec::new();
                    
                    for i in 0..concurrency {
                        let service = ctx.casbin_service.clone();
                        let resource = format!("/api/v1/resource{}", i % 10);
                        
                        let handle = tokio::spawn(async move {
                            service.enforce("admin_user", &resource, "GET").await.unwrap()
                        });
                        
                        handles.push(handle);
                    }
                    
                    let results: Vec<bool> = futures::future::join_all(handles)
                        .await
                        .into_iter()
                        .map(|r| r.unwrap())
                        .collect();
                    
                    black_box(results)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark policy operations (add/remove)
fn bench_policy_operations(c: &mut Criterion) {
    let ctx = BenchmarkContext::new();
    
    let mut group = c.benchmark_group("policy_operations");
    
    group.bench_function("add_single_policy", |b| {
        let mut counter = 0;
        b.to_async(&ctx.rt).iter(|| async {
            counter += 1;
            let subject = format!("bench_user_{}", counter);
            let resource = format!("/api/v1/bench_resource_{}", counter);
            
            black_box(
                ctx.casbin_service
                    .add_policy(&subject, &resource, "GET")
                    .await
                    .unwrap()
            )
        })
    });
    
    group.bench_function("remove_single_policy", |b| {
        let mut counter = 0;
        b.to_async(&ctx.rt).iter_batched(
            || {
                counter += 1;
                let subject = format!("remove_user_{}", counter);
                let resource = format!("/api/v1/remove_resource_{}", counter);
                
                // Setup: Add policy first
                ctx.rt.block_on(async {
                    ctx.casbin_service.add_policy(&subject, &resource, "GET").await.unwrap();
                });
                
                (subject, resource)
            },
            |(subject, resource)| async {
                black_box(
                    ctx.casbin_service
                        .remove_policy(&subject, &resource, "GET")
                        .await
                        .unwrap()
                )
            },
            criterion::BatchSize::SmallInput,
        )
    });
    
    // Benchmark batch policy operations
    let batch_sizes = vec![5, 10, 25, 50, 100];
    
    for &batch_size in &batch_sizes {
        group.bench_with_input(
            BenchmarkId::new("batch_add_policies", batch_size),
            &batch_size,
            |b, &batch_size| {
                let mut counter = 0;
                b.to_async(&ctx.rt).iter(|| async {
                    counter += 1;
                    let policies: Vec<(String, String, String)> = (0..batch_size)
                        .map(|i| {
                            (
                                format!("batch_user_{}_{}", counter, i),
                                format!("/api/v1/batch_resource_{}_{}", counter, i),
                                "GET".to_string(),
                            )
                        })
                        .collect();
                    
                    black_box(
                        ctx.casbin_service
                            .add_policies(policies)
                            .await
                            .unwrap()
                    )
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark cache performance
fn bench_cache_performance(c: &mut Criterion) {
    let ctx = BenchmarkContext::new();
    
    let mut group = c.benchmark_group("cache_performance");
    
    // Benchmark cache hit ratio under load
    group.bench_function("cache_hit_pattern", |b| {
        b.to_async(&ctx.rt).iter(|| async {
            // Create a pattern of requests that should benefit from caching
            let requests = vec![
                ("user1", "/api/v1/resource1", "GET"),
                ("user1", "/api/v1/resource2", "GET"),
                ("user2", "/api/v1/resource1", "GET"),
                ("user1", "/api/v1/resource1", "GET"), // Cache hit
                ("user2", "/api/v1/resource2", "GET"),
                ("user1", "/api/v1/resource2", "GET"), // Cache hit
                ("user3", "/api/v1/resource1", "GET"),
                ("user1", "/api/v1/resource1", "GET"), // Cache hit
            ];
            
            let mut results = Vec::new();
            for (user, resource, action) in requests {
                let result = ctx.casbin_service.enforce(user, resource, action).await.unwrap();
                results.push(result);
            }
            
            black_box(results)
        })
    });
    
    // Benchmark cache invalidation performance
    group.bench_function("cache_invalidation", |b| {
        b.to_async(&ctx.rt).iter(|| async {
            // Fill cache with some entries
            for i in 0..10 {
                ctx.casbin_service
                    .enforce("cache_user", &format!("/api/v1/resource{}", i), "GET")
                    .await
                    .unwrap();
            }
            
            // Add a new policy (triggers cache invalidation)
            black_box(
                ctx.casbin_service
                    .add_policy("cache_user", "/api/v1/new_resource", "GET")
                    .await
                    .unwrap()
            )
        })
    });
    
    group.finish();
}

/// Benchmark wildcard pattern matching performance
fn bench_wildcard_patterns(c: &mut Criterion) {
    let ctx = BenchmarkContext::new();
    
    let mut group = c.benchmark_group("wildcard_patterns");
    
    // Setup wildcard policies
    ctx.rt.block_on(async {
        ctx.casbin_service.add_policy("wildcard_user", "/api/v1/*", "GET").await.unwrap();
        ctx.casbin_service.add_policy("action_wildcard_user", "/api/v1/specific", "*").await.unwrap();
    });
    
    group.bench_function("resource_wildcard", |b| {
        b.to_async(&ctx.rt).iter(|| async {
            black_box(
                ctx.casbin_service
                    .enforce("wildcard_user", "/api/v1/random_resource", "GET")
                    .await
                    .unwrap()
            )
        })
    });
    
    group.bench_function("action_wildcard", |b| {
        b.to_async(&ctx.rt).iter(|| async {
            black_box(
                ctx.casbin_service
                    .enforce("action_wildcard_user", "/api/v1/specific", "POST")
                    .await
                    .unwrap()
            )
        })
    });
    
    group.bench_function("no_wildcard", |b| {
        b.to_async(&ctx.rt).iter(|| async {
            black_box(
                ctx.casbin_service
                    .enforce("admin_user", "/api/v1/resource1", "GET")
                    .await
                    .unwrap()
            )
        })
    });
    
    group.finish();
}

/// Benchmark memory usage and cleanup
fn bench_memory_usage(c: &mut Criterion) {
    let ctx = BenchmarkContext::new();
    
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(10); // Fewer samples for memory tests
    
    group.bench_function("policy_reload", |b| {
        b.to_async(&ctx.rt).iter(|| async {
            // This forces a complete reload of policies from database
            black_box(
                ctx.casbin_service
                    .reload_policies()
                    .await
                    .unwrap()
            )
        })
    });
    
    group.bench_function("cache_clear_and_rebuild", |b| {
        b.to_async(&ctx.rt).iter(|| async {
            // Clear cache
            ctx.casbin_service.clear_cache().await;
            
            // Rebuild cache with some enforcement calls
            for i in 0..10 {
                ctx.casbin_service
                    .enforce("admin_user", &format!("/api/v1/resource{}", i), "GET")
                    .await
                    .unwrap();
            }
            
            black_box(())
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_single_enforcement,
    bench_role_complexity,
    bench_concurrent_enforcement,
    bench_policy_operations,
    bench_cache_performance,
    bench_wildcard_patterns,
    bench_memory_usage
);

criterion_main!(benches);