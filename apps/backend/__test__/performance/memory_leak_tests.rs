use std::sync::Arc;

use std::time::{ Duration, Instant };

use tokio::time::sleep;

use tokio::sync::{ Semaphore, RwLock };

use std::collections::HashMap;

use epsx::infra::container::AppContainer;

use epsx::core::types::{ UserId, SessionId };

use std::sync::atomic::{ AtomicUsize, Ordering };

use sysinfo::{ System, SystemExt, ProcessExt, PidExt };

/// Memory leak detection and resource utilization tests
///
/// These tests validate that the middleware stack doesn't have memory leaks
/// and properly manages system resources under sustained load.

#[tokio::test]
async fn test_memory_leak_detection() {
  let container = Arc::new(AppContainer::new_test().await.unwrap());
  let initial_memory = get_memory_usage();

  println!("Initial memory usage: {} MB", initial_memory);

  // Run intensive operations for 30 seconds
  let duration = Duration::from_secs(30);
  let start_time = Instant::now();
  let mut iteration = 0;

  while start_time.elapsed() < duration {
    // Simulate typical request processing
    let user_id = UserId::new(format!("memory-test-user-{}", iteration));
    let sid = SessionId::new(Uuid::new_v4().to_string());

    // Perform operations that could potentially leak memory
    let user_repo = container.user_repo();
    let permission_service = container.permission_service();
    let cache = container.cache();

    // User operations
    let _ = user_repo.find_by_id(&user_id).await;
    let _ = permission_service.get_user_permissions(&user_id).await;

    // Cache operations
    let cache_key = format!("memory_test:{}", iteration);
    let cache_value =
      serde_json::json!({
            "test_data": format!("data_for_iteration_{}", iteration),
            "timestamp": chrono::Utc::now(),
            "large_array": (0..100).map(|i| format!("item_{}", i)).collect::<Vec<_>>()
        });

    let _ = cache.set(
      &cache_key,
      &cache_value,
      Some(Duration::from_secs(10))
    ).await;
    let _ = cache.get::<serde_json::Value>(&cache_key).await;
    let _ = cache.delete(&cache_key).await;

    iteration += 1;

    // Check memory every 1000 iterations
    if iteration % 1000 == 0 {
      let current_memory = get_memory_usage();
      println!("Memory after {} iterations: {} MB", iteration, current_memory);

      // Allow some memory growth but detect excessive leaks
      let memory_growth = (current_memory as f64) / (initial_memory as f64);
      assert!(
        memory_growth < 3.0,
        "Memory usage grew too much: {}x from {} MB to {} MB",
        memory_growth,
        initial_memory,
        current_memory
      );
    }

    // Small delay to prevent overwhelming the system
    if iteration % 100 == 0 {
      tokio::task::yield_now().await;
    }
  }

  // Force garbage collection and check final memory
  std::hint::black_box(&container);
  tokio::time::sleep(Duration::from_millis(100)).await;

  let final_memory = get_memory_usage();
  println!("Final memory usage: {} MB", final_memory);
  println!("Total iterations: {}", iteration);

  // Final memory should not be excessively higher than initial
  let memory_growth = (final_memory as f64) / (initial_memory as f64);
  assert!(
    memory_growth < 2.5,
    "Final memory usage grew too much: {}x from {} MB to {} MB",
    memory_growth,
    initial_memory,
    final_memory
  );
}

#[tokio::test]
async fn test_connection_pool_resource_cleanup() {
  let container = Arc::new(AppContainer::new_test().await.unwrap());
  let initial_memory = get_memory_usage();

  // Test connection pool doesn't leak connections
  let concurrent_operations = 50;
  let iterations = 100;
  let semaphore = Arc::new(Semaphore::new(concurrent_operations));

  for round in 0..iterations {
    let mut handles = Vec::new();

    for i in 0..concurrent_operations {
      let container = container.clone();
      let semaphore = semaphore.clone();
      let user_id = format!("pool-test-{}-{}", round, i);

      handles.push(
        tokio::spawn(async move {
          let _permit = semaphore.acquire().await.unwrap();

          let user_repo = container.user_repo();
          let permission_service = container.permission_service();

          // Multiple database operations to stress connection pool
          let user_id = UserId::new(user_id);
          let _ = user_repo.find_by_id(&user_id).await;
          let _ = permission_service.get_user_permissions(&user_id).await;
          let _ = permission_service.check_permission(
            &user_id,
            "READ_PROFILE"
          ).await;
        })
      );
    }

    // Wait for all operations to complete
    for handle in handles {
      handle.await.unwrap();
    }

    // Check memory periodically
    if round % 20 == 0 {
      let current_memory = get_memory_usage();
      println!(
        "Memory after {} rounds of connection pool tests: {} MB",
        round,
        current_memory
      );

      // Connection pool should not cause excessive memory growth
      let memory_growth = (current_memory as f64) / (initial_memory as f64);
      assert!(
        memory_growth < 2.0,
        "Connection pool caused excessive memory growth: {}x",
        memory_growth
      );
    }

    // Brief pause between rounds
    tokio::time::sleep(Duration::from_millis(10)).await;
  }

  let final_memory = get_memory_usage();
  println!("Final memory after connection pool tests: {} MB", final_memory);

  // Ensure connections are properly cleaned up
  let memory_growth = (final_memory as f64) / (initial_memory as f64);
  assert!(
    memory_growth < 1.8,
    "Connection pool leaked memory: {}x growth",
    memory_growth
  );
}

#[tokio::test]
async fn test_cache_memory_management() {
  let container = Arc::new(AppContainer::new_test().await.unwrap());
  let cache = container.cache();
  let initial_memory = get_memory_usage();

  // Test cache memory management with large values
  let cache_entries = 1000;
  let mut cache_keys = Vec::new();

  // Fill cache with large entries
  for i in 0..cache_entries {
    let key = format!("cache_memory_test:{}", i);
    let large_value =
      serde_json::json!({
            "index": i,
            "large_data": (0..1000).map(|j| format!("data_{}_{}", i, j)).collect::<Vec<_>>(),
            "metadata": {
                "created_at": chrono::Utc::now(),
                "size": "large",
                "test_type": "memory_management"
            }
        });

    cache
      .set(&key, &large_value, Some(Duration::from_minutes(5))).await
      .unwrap();
    cache_keys.push(key);

    // Check memory growth periodically
    if i % 100 == 0 {
      let current_memory = get_memory_usage();
      println!("Memory after {} cache entries: {} MB", i, current_memory);
    }
  }

  let peak_memory = get_memory_usage();
  println!(
    "Peak memory with {} cache entries: {} MB",
    cache_entries,
    peak_memory
  );

  // Read all cache entries to ensure they're accessible
  for key in &cache_keys {
    let _value: Option<serde_json::Value> = cache.get(key).await.unwrap();
  }

  // Clear half the cache entries
  for i in 0..cache_entries / 2 {
    cache.delete(&cache_keys[i]).await.unwrap();
  }

  // Give cache time to clean up
  tokio::time::sleep(Duration::from_millis(500)).await;

  let after_partial_cleanup = get_memory_usage();
  println!("Memory after partial cache cleanup: {} MB", after_partial_cleanup);

  // Clear remaining entries
  for i in cache_entries / 2..cache_entries {
    cache.delete(&cache_keys[i]).await.unwrap();
  }

  // Give cache time to fully clean up
  tokio::time::sleep(Duration::from_secs(1)).await;

  let final_memory = get_memory_usage();
  println!("Final memory after full cache cleanup: {} MB", final_memory);

  // Memory should return close to initial levels
  let memory_retention = (final_memory as f64) / (initial_memory as f64);
  assert!(
    memory_retention < 1.5,
    "Cache cleanup didn't free enough memory: {}x retention",
    memory_retention
  );
}

#[tokio::test]
async fn test_concurrent_memory_allocation() {
  let container = Arc::new(AppContainer::new_test().await.unwrap());
  let initial_memory = get_memory_usage();

  // Test memory allocation patterns under high concurrency
  let concurrent_tasks = 100;
  let allocations_per_task = 50;

  let mut handles = Vec::new();
  let allocation_counter = Arc::new(AtomicUsize::new(0));

  for task_id in 0..concurrent_tasks {
    let container = container.clone();
    let counter = allocation_counter.clone();

    handles.push(
      tokio::spawn(async move {
        for allocation_id in 0..allocations_per_task {
          let user_id = UserId::new(
            format!("concurrent-{}-{}", task_id, allocation_id)
          );

          // Simulate memory-intensive operations
          let cache = container.cache();
          let permission_service = container.permission_service();

          // Large data structure allocation
          let large_data = (0..500)
            .map(
              |i|
                serde_json::json!({
                    "id": i,
                    "task": task_id,
                    "allocation": allocation_id,
                    "data": format!("test_data_{}_{}_{}", task_id, allocation_id, i),
                    "timestamp": chrono::Utc::now()
                })
            )
            .collect::<Vec<_>>();

          // Cache the large data
          let cache_key = format!("concurrent:{}:{}", task_id, allocation_id);
          let _ = cache.set(
            &cache_key,
            &serde_json::Value::Array(large_data),
            Some(Duration::from_secs(1))
          ).await;

          // Perform some operations
          let _ = permission_service.get_user_permissions(&user_id).await;

          // Clean up immediately
          let _ = cache.delete(&cache_key).await;

          counter.fetch_add(1, Ordering::Relaxed);

          // Yield to prevent starving other tasks
          if allocation_id % 10 == 0 {
            tokio::task::yield_now().await;
          }
        }
      })
    );
  }

  // Wait for all tasks to complete
  for handle in handles {
    handle.await.unwrap();
  }

  let total_allocations = allocation_counter.load(Ordering::Relaxed);
  let final_memory = get_memory_usage();

  println!("Completed {} concurrent allocations", total_allocations);
  println!("Final memory usage: {} MB", final_memory);

  // Memory usage should be reasonable after cleanup
  let memory_growth = (final_memory as f64) / (initial_memory as f64);
  assert!(
    memory_growth < 2.0,
    "Concurrent allocations caused excessive memory growth: {}x",
    memory_growth
  );

  assert_eq!(
    total_allocations,
    concurrent_tasks * allocations_per_task,
    "Not all allocations completed successfully"
  );
}

#[tokio::test]
async fn test_session_memory_lifecycle() {
  let container = Arc::new(AppContainer::new_test().await.unwrap());
  let initial_memory = get_memory_usage();

  // Test session creation and cleanup doesn't leak memory
  let session_count = 500;
  let mut session_data = Vec::new();

  // Create many sessions
  for i in 0..session_count {
    let user_id = UserId::new(format!("session-user-{}", i));
    let sid = SessionId::new(Uuid::new_v4().to_string());

    // Store session data in cache (simulating session storage)
    let cache = container.cache();
    let session_key = format!("session:{}", sid);
    let session_value =
      serde_json::json!({
            "user_id": user_id,
            "created_at": chrono::Utc::now(),
            "permissions": ["READ_PROFILE", "TRADE_STOCKS"],
            "metadata": {
                "ip": format!("192.168.1.{}", i % 255),
                "user_agent": "Test User Agent",
                "session_data": (0..50).map(|j| format!("session_data_{}_{}", i, j)).collect::<Vec<_>>()
            }
        });

    cache
      .set(&session_key, &session_value, Some(Duration::from_minutes(30))).await
      .unwrap();
    session_data.push((sid, session_key));

    // Check memory periodically
    if i % 100 == 0 {
      let current_memory = get_memory_usage();
      println!("Memory after {} sessions: {} MB", i, current_memory);
    }
  }

  let peak_memory = get_memory_usage();
  println!("Peak memory with {} sessions: {} MB", session_count, peak_memory);

  // Validate sessions exist
  let cache = container.cache();
  let mut valid_sessions = 0;
  for (_, session_key) in &session_data {
    let session: Option<serde_json::Value> = cache
      .get(session_key).await
      .unwrap();
    if session.is_some() {
      valid_sessions += 1;
    }
  }

  println!("Valid sessions found: {}", valid_sessions);
  assert!(
    valid_sessions > (session_count * 95) / 100,
    "Too many sessions were lost: {}/{}",
    valid_sessions,
    session_count
  );

  // Clean up sessions
  for (_, session_key) in &session_data {
    cache.delete(session_key).await.unwrap();
  }

  // Give time for cleanup
  tokio::time::sleep(Duration::from_millis(500)).await;

  let final_memory = get_memory_usage();
  println!("Final memory after session cleanup: {} MB", final_memory);

  // Memory should return to reasonable levels
  let memory_retention = (final_memory as f64) / (initial_memory as f64);
  assert!(
    memory_retention < 1.6,
    "Session cleanup didn't free enough memory: {}x retention",
    memory_retention
  );
}

#[tokio::test]
async fn test_resource_handle_cleanup() {
  let container = Arc::new(AppContainer::new_test().await.unwrap());
  let initial_memory = get_memory_usage();

  // Test that various resource handles are properly cleaned up
  let iterations = 200;
  let resources_per_iteration = 20;

  for iteration in 0..iterations {
    let mut handles = Vec::new();
    let shared_data = Arc::new(RwLock::new(HashMap::<String, Vec<u8>>::new()));

    // Create multiple async tasks that use various resources
    for i in 0..resources_per_iteration {
      let container = container.clone();
      let data = shared_data.clone();
      let key = format!("resource_{}_{}", iteration, i);

      handles.push(
        tokio::spawn(async move {
          // Database resources
          let user_repo = container.user_repo();
          let permission_service = container.permission_service();
          let cache = container.cache();

          let user_id = UserId::new(
            format!("resource-user-{}-{}", iteration, i)
          );

          // Use multiple resource types
          let _ = user_repo.find_by_id(&user_id).await;
          let _ = permission_service.get_user_permissions(&user_id).await;

          // Cache operations
          let cache_key = format!("resource:{}:{}", iteration, i);
          let cache_data = vec![0u8; 1024]; // 1KB of data
          let _ = cache.set(
            &cache_key,
            &cache_data,
            Some(Duration::from_secs(5))
          ).await;

          // Shared memory operations
          {
            let mut guard = data.write().await;
            guard.insert(key.clone(), vec![0u8; 512]);
          }

          // Simulate some work
          tokio::time::sleep(Duration::from_micros(100)).await;

          // Clean up explicitly
          let _ = cache.delete(&cache_key).await;
          {
            let mut guard = data.write().await;
            guard.remove(&key);
          }
        })
      );
    }

    // Wait for all tasks to complete
    for handle in handles {
      handle.await.unwrap();
    }

    // Verify shared data is cleaned up
    {
      let guard = shared_data.read().await;
      assert!(
        guard.is_empty(),
        "Shared data not cleaned up properly in iteration {}",
        iteration
      );
    }

    // Check memory periodically
    if iteration % 50 == 0 {
      let current_memory = get_memory_usage();
      println!(
        "Memory after {} resource iterations: {} MB",
        iteration,
        current_memory
      );

      let memory_growth = (current_memory as f64) / (initial_memory as f64);
      assert!(
        memory_growth < 2.5,
        "Resource handles caused excessive memory growth: {}x",
        memory_growth
      );
    }

    // Brief pause between iterations
    if iteration % 10 == 0 {
      tokio::task::yield_now().await;
    }
  }

  let final_memory = get_memory_usage();
  println!("Final memory after resource handle tests: {} MB", final_memory);

  let memory_growth = (final_memory as f64) / (initial_memory as f64);
  assert!(
    memory_growth < 2.0,
    "Resource handle cleanup failed: {}x memory growth",
    memory_growth
  );
}

#[tokio::test]
async fn test_sustained_load_memory_stability() {
  let container = Arc::new(AppContainer::new_test().await.unwrap());
  let initial_memory = get_memory_usage();

  // Test memory stability under sustained load
  let duration = Duration::from_secs(60); // 1 minute sustained test
  let start_time = Instant::now();
  let mut operation_count = 0;
  let mut memory_samples = Vec::new();

  println!("Starting sustained load test for {} seconds", duration.as_secs());

  while start_time.elapsed() < duration {
    // Mix of different operations
    let operation_type = operation_count % 4;

    match operation_type {
      0 => {
        // User operations
        let user_id = UserId::new(
          format!("sustained-user-{}", operation_count)
        );
        let user_repo = container.user_repo();
        let _ = user_repo.find_by_id(&user_id).await;
      }
      1 => {
        // Permission operations
        let user_id = UserId::new(
          format!("sustained-perm-{}", operation_count)
        );
        let permission_service = container.permission_service();
        let _ = permission_service.get_user_permissions(&user_id).await;
      }
      2 => {
        // Cache operations
        let cache = container.cache();
        let key = format!("sustained:{}", operation_count);
        let value =
          serde_json::json!({
                    "operation": operation_count,
                    "timestamp": chrono::Utc::now(),
                    "data": (0..20).map(|i| format!("data_{}", i)).collect::<Vec<_>>()
                });
        let _ = cache.set(&key, &value, Some(Duration::from_secs(30))).await;
        let _ = cache.get::<serde_json::Value>(&key).await;
        let _ = cache.delete(&key).await;
      }
      3 => {
        // Mixed operations
        let user_id = UserId::new(
          format!("sustained-mixed-{}", operation_count)
        );
        let cache = container.cache();
        let permission_service = container.permission_service();

        let cache_key = format!("mixed:{}", operation_count);
        let _ = cache.set(
          &cache_key,
          &operation_count,
          Some(Duration::from_secs(10))
        ).await;
        let _ = permission_service.check_permission(
          &user_id,
          "READ_PROFILE"
        ).await;
        let _ = cache.delete(&cache_key).await;
      }
      _ => unreachable!(),
    }

    operation_count += 1;

    // Sample memory every 1000 operations
    if operation_count % 1000 == 0 {
      let current_memory = get_memory_usage();
      memory_samples.push(current_memory);

      println!("Operation {}: {} MB", operation_count, current_memory);

      // Check for excessive memory growth
      let memory_growth = (current_memory as f64) / (initial_memory as f64);
      assert!(
        memory_growth < 3.0,
        "Excessive memory growth during sustained load: {}x at operation {}",
        memory_growth,
        operation_count
      );
    }

    // Yield periodically to prevent blocking
    if operation_count % 100 == 0 {
      tokio::task::yield_now().await;
    }
  }

  let final_memory = get_memory_usage();
  println!("Sustained load test completed:");
  println!("  Operations: {}", operation_count);
  println!("  Initial memory: {} MB", initial_memory);
  println!("  Final memory: {} MB", final_memory);
  println!("  Memory samples: {:?}", memory_samples);

  // Analyze memory stability
  if memory_samples.len() > 1 {
    let max_memory = memory_samples.iter().max().unwrap();
    let min_memory = memory_samples.iter().min().unwrap();
    let memory_variance = (*max_memory as f64) / (*min_memory as f64);

    println!(
      "  Memory variance: {}x (max: {} MB, min: {} MB)",
      memory_variance,
      max_memory,
      min_memory
    );

    // Memory should be relatively stable
    assert!(
      memory_variance < 2.0,
      "Memory usage too unstable: {}x variance",
      memory_variance
    );
  }

  // Final memory should be reasonable
  let final_growth = (final_memory as f64) / (initial_memory as f64);
  assert!(
    final_growth < 2.5,
    "Final memory growth too high: {}x",
    final_growth
  );
}

// Helper function to get current memory usage in MB
fn get_memory_usage() -> u64 {
  let mut system = System::new_all();
  system.refresh_all();

  if let Some(process) = system.process(sysinfo::get_current_pid().unwrap()) {
    process.memory() / 1024 / 1024 // Convert to MB
  } else {
    0
  }
}
