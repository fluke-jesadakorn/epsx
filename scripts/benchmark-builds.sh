#!/bin/bash

# EPSX Build Performance Benchmarking and Monitoring
# Tracks build performance across different container engines and optimizations
# Helps measure OrbStack vs Docker Desktop performance improvements

set -e

# Configuration
BENCHMARK_LOG_FILE="./logs/build-performance.log"
BENCHMARK_DATA_DIR="./logs/benchmarks"
CURRENT_DATE=$(date '+%Y-%m-%d')
CURRENT_TIME=$(date '+%H:%M:%S')

# Create directories if they don't exist
mkdir -p "$(dirname "$BENCHMARK_LOG_FILE")"
mkdir -p "$BENCHMARK_DATA_DIR"

echo "📊 EPSX Build Performance Benchmark"
echo "==================================="
echo "📅 Date: $CURRENT_DATE"
echo "⏰ Time: $CURRENT_TIME"
echo ""

# Detect container engine
detect_container_engine() {
    if docker info 2>/dev/null | grep -q "OrbStack"; then
        echo "orbstack"
    elif docker info 2>/dev/null | grep -q "Docker Desktop"; then
        echo "docker-desktop"
    elif command -v podman &> /dev/null; then
        echo "podman"
    else
        echo "unknown"
    fi
}

# Get system information
get_system_info() {
    echo "🖥️  System Information:"
    echo "   OS: $(uname -s)"
    echo "   Architecture: $(uname -m)"
    echo "   Kernel: $(uname -r)"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS specific info
        cpu_model=$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "Unknown")
        memory_gb=$(echo "scale=1; $(sysctl -n hw.memsize) / 1024 / 1024 / 1024" | bc 2>/dev/null || echo "Unknown")
        echo "   CPU: $cpu_model"
        echo "   Memory: ${memory_gb}GB"
        
        # Battery status if available
        if command -v pmset &> /dev/null; then
            battery_info=$(pmset -g batt | grep -o '[0-9]*%' | head -1 || echo "N/A")
            power_source=$(pmset -g batt | grep -o 'AC Power\|Battery Power' | head -1 || echo "Unknown")
            echo "   Battery: $battery_info ($power_source)"
        fi
    else
        # Linux specific info
        if [ -f /proc/cpuinfo ]; then
            cpu_model=$(grep "model name" /proc/cpuinfo | head -1 | cut -d: -f2 | xargs)
            echo "   CPU: $cpu_model"
        fi
        if [ -f /proc/meminfo ]; then
            memory_kb=$(grep "MemTotal" /proc/meminfo | awk '{print $2}')
            memory_gb=$(echo "scale=1; $memory_kb / 1024 / 1024" | bc 2>/dev/null || echo "Unknown")
            echo "   Memory: ${memory_gb}GB"
        fi
    fi
    
    echo ""
}

# Container startup benchmark
benchmark_container_startup() {
    local engine=$1
    echo "⚡ Benchmarking container startup performance..."
    
    local total_time=0
    local iterations=5
    
    for i in $(seq 1 $iterations); do
        echo "   Test $i/$iterations..."
        
        start_time=$(date +%s%N)
        docker run --rm alpine:latest echo "Benchmark test $i" >/dev/null 2>&1
        end_time=$(date +%s%N)
        
        duration_ms=$(( (end_time - start_time) / 1000000 ))
        total_time=$((total_time + duration_ms))
        
        echo "      Duration: ${duration_ms}ms"
    done
    
    average_time=$((total_time / iterations))
    echo "   📊 Average startup time: ${average_time}ms ($iterations tests)"
    
    # Performance rating
    if [ "$average_time" -lt 2000 ]; then
        rating="Excellent (OrbStack-level)"
    elif [ "$average_time" -lt 5000 ]; then
        rating="Very Good"
    elif [ "$average_time" -lt 15000 ]; then
        rating="Good"
    elif [ "$average_time" -lt 30000 ]; then
        rating="Fair"
    else
        rating="Poor (Docker Desktop-level)"
    fi
    
    echo "   🏆 Performance Rating: $rating"
    echo ""
    
    return $average_time
}

# EPSX build benchmark
benchmark_epsx_build() {
    local engine=$1
    local build_type=${2:-"incremental"}
    
    echo "🏗️  Benchmarking EPSX $build_type build..."
    
    # Clean build vs incremental build
    if [ "$build_type" = "clean" ]; then
        echo "   🧹 Performing clean build (no cache)..."
        rm -rf /tmp/.buildx-cache-orbstack 2>/dev/null || true
        docker system prune -f >/dev/null 2>&1 || true
    else
        echo "   🔄 Performing incremental build (with cache)..."
    fi
    
    # Record system state before build
    cpu_before=$(ps aux | grep -E "(docker|orbstack|podman)" | grep -v grep | awk '{sum += $3} END {printf "%.1f", sum+0}')
    
    start_time=$(date +%s)
    
    # Run the build
    if ./scripts/build.sh >/dev/null 2>&1; then
        end_time=$(date +%s)
        build_duration=$((end_time - start_time))
        
        # Record system state after build
        cpu_after=$(ps aux | grep -E "(docker|orbstack|podman)" | grep -v grep | awk '{sum += $3} END {printf "%.1f", sum+0}')
        
        echo "   ✅ Build completed successfully"
        echo "   ⏱️  Build time: ${build_duration}s"
        echo "   🖥️  CPU usage: ${cpu_before}% → ${cpu_after}%"
        
        # Get image sizes
        frontend_size=$(docker images --format "{{.Size}}" asia-southeast1-docker.pkg.dev/your-project-id/epsx/frontend:latest 2>/dev/null || echo "N/A")
        admin_size=$(docker images --format "{{.Size}}" asia-southeast1-docker.pkg.dev/your-project-id/epsx/admin-frontend:latest 2>/dev/null || echo "N/A")
        backend_size=$(docker images --format "{{.Size}}" asia-southeast1-docker.pkg.dev/your-project-id/epsx/backend:latest 2>/dev/null || echo "N/A")
        
        echo "   📦 Image sizes:"
        echo "      Frontend: $frontend_size"
        echo "      Admin: $admin_size"
        echo "      Backend: $backend_size"
        
        # Performance rating based on build time
        if [ "$build_duration" -lt 180 ]; then
            build_rating="Excellent (OrbStack optimized)"
        elif [ "$build_duration" -lt 300 ]; then
            build_rating="Very Good"
        elif [ "$build_duration" -lt 600 ]; then
            build_rating="Good"
        elif [ "$build_duration" -lt 1200 ]; then
            build_rating="Fair"
        else
            build_rating="Poor (needs optimization)"
        fi
        
        echo "   🏆 Build Performance: $build_rating"
        echo ""
        
        return $build_duration
    else
        echo "   ❌ Build failed"
        return 9999
    fi
}

# Save benchmark results
save_benchmark_results() {
    local engine=$1
    local startup_time=$2
    local build_time=$3
    local build_type=$4
    
    local benchmark_file="${BENCHMARK_DATA_DIR}/benchmark-${CURRENT_DATE}.json"
    
    # Create JSON entry
    local json_entry=$(cat <<EOF
{
  "timestamp": "$(date -Iseconds)",
  "date": "$CURRENT_DATE",
  "time": "$CURRENT_TIME",
  "container_engine": "$engine",
  "build_type": "$build_type",
  "startup_time_ms": $startup_time,
  "build_time_seconds": $build_time,
  "system_info": {
    "os": "$(uname -s)",
    "arch": "$(uname -m)",
    "kernel": "$(uname -r)"
  }
}
EOF
)
    
    # Append to benchmark file
    if [ -f "$benchmark_file" ]; then
        # Remove last ] and add comma
        sed -i.bak '$ s/]/,/' "$benchmark_file"
        echo "$json_entry" >> "$benchmark_file"
        echo "]" >> "$benchmark_file"
    else
        # Create new file
        echo "[" > "$benchmark_file"
        echo "$json_entry" >> "$benchmark_file"
        echo "]" >> "$benchmark_file"
    fi
    
    # Also append to log file
    echo "[$CURRENT_DATE $CURRENT_TIME] Engine: $engine, Build: $build_type, Startup: ${startup_time}ms, Build: ${build_time}s" >> "$BENCHMARK_LOG_FILE"
}

# Compare with historical data
compare_performance() {
    local current_startup=$1
    local current_build=$2
    local engine=$3
    
    echo "📈 Performance Comparison:"
    
    # Find previous benchmarks
    local prev_data=$(grep "$engine" "$BENCHMARK_LOG_FILE" 2>/dev/null | tail -2 | head -1 || echo "")
    
    if [ -n "$prev_data" ]; then
        local prev_startup=$(echo "$prev_data" | grep -o 'Startup: [0-9]*ms' | grep -o '[0-9]*')
        local prev_build=$(echo "$prev_data" | grep -o 'Build: [0-9]*s' | grep -o '[0-9]*')
        
        if [ -n "$prev_startup" ] && [ -n "$prev_build" ]; then
            startup_diff=$((current_startup - prev_startup))
            build_diff=$((current_build - prev_build))
            
            echo "   ⚡ Container Startup:"
            echo "      Previous: ${prev_startup}ms"
            echo "      Current: ${current_startup}ms"
            if [ $startup_diff -lt 0 ]; then
                echo "      Change: ${startup_diff}ms (🌟 ${startup_diff#-}ms faster)"
            elif [ $startup_diff -gt 0 ]; then
                echo "      Change: +${startup_diff}ms (⚠️ slower)"
            else
                echo "      Change: No change"
            fi
            
            echo "   🏗️  Build Time:"
            echo "      Previous: ${prev_build}s"
            echo "      Current: ${current_build}s"
            if [ $build_diff -lt 0 ]; then
                echo "      Change: ${build_diff}s (🌟 ${build_diff#-}s faster)"
            elif [ $build_diff -gt 0 ]; then
                echo "      Change: +${build_diff}s (⚠️ slower)"
            else
                echo "      Change: No change"
            fi
        fi
    else
        echo "   📊 No previous data for comparison"
        echo "   ℹ️  This is your baseline measurement"
    fi
    
    echo ""
}

# Generate performance report
generate_report() {
    echo "📋 Performance Summary Report:"
    echo "   📅 Date: $CURRENT_DATE"
    echo "   🖥️  Container Engine: $CONTAINER_ENGINE"
    echo "   ⚡ Startup Performance: ${startup_result}ms"
    echo "   🏗️  Build Performance: ${build_result}s"
    echo ""
    
    # Recommendations based on results
    echo "💡 Recommendations:"
    
    if [ "$CONTAINER_ENGINE" = "docker-desktop" ]; then
        echo "   🚀 Migrate to OrbStack for significant improvements:"
        echo "      • Expected startup improvement: 15x faster"
        echo "      • Expected build improvement: 3-4x faster"
        echo "      • Battery life improvement: 4x longer"
        echo "      • Guide: ./scripts/orbstack-migration-guide.md"
    elif [ "$CONTAINER_ENGINE" = "orbstack" ]; then
        if [ $startup_result -lt 3000 ] && [ $build_result -lt 300 ]; then
            echo "   🌟 Excellent performance! You're getting optimal results from OrbStack."
        else
            echo "   🔧 Consider these optimizations:"
            echo "      • Check build cache configuration"
            echo "      • Verify system resources aren't constrained"
            echo "      • Review Dockerfile optimizations"
        fi
    elif [ "$CONTAINER_ENGINE" = "podman" ]; then
        echo "   🔧 Podman optimizations:"
        echo "      • Verify machine configuration for ARM64 support"
        echo "      • Consider OrbStack for development speed"
        echo "      • Leverage Podman's security benefits for production"
    fi
    
    echo ""
    echo "📊 Benchmark data saved to: $BENCHMARK_DATA_DIR"
    echo "📝 Performance log: $BENCHMARK_LOG_FILE"
}

# Main benchmark execution
main() {
    local build_type=${1:-"incremental"}
    
    # System information
    get_system_info
    
    # Detect container engine
    CONTAINER_ENGINE=$(detect_container_engine)
    echo "🔍 Container Engine: $CONTAINER_ENGINE"
    echo ""
    
    # Run benchmarks
    benchmark_container_startup "$CONTAINER_ENGINE"
    startup_result=$?
    
    benchmark_epsx_build "$CONTAINER_ENGINE" "$build_type"
    build_result=$?
    
    # Save results
    save_benchmark_results "$CONTAINER_ENGINE" "$startup_result" "$build_result" "$build_type"
    
    # Compare with historical data
    compare_performance "$startup_result" "$build_result" "$CONTAINER_ENGINE"
    
    # Generate report
    generate_report
    
    echo "🎯 Benchmark completed! Check logs for detailed performance tracking."
}

# Help function
show_help() {
    echo "Usage: $0 [build_type]"
    echo ""
    echo "Build types:"
    echo "  incremental  - Build with existing cache (default)"
    echo "  clean        - Clean build without cache"
    echo ""
    echo "Examples:"
    echo "  $0                # Incremental build benchmark"
    echo "  $0 clean         # Clean build benchmark"
    echo "  $0 incremental   # Explicit incremental build"
    echo ""
    echo "The script will:"
    echo "  1. Test container startup performance"
    echo "  2. Benchmark EPSX build process"
    echo "  3. Compare with historical data"
    echo "  4. Generate performance recommendations"
    echo ""
    echo "Results are saved to:"
    echo "  • JSON data: $BENCHMARK_DATA_DIR"
    echo "  • Log file: $BENCHMARK_LOG_FILE"
}

# Handle command line arguments
case "${1:-}" in
    -h|--help|help)
        show_help
        exit 0
        ;;
    clean|incremental|"")
        main "${1:-incremental}"
        ;;
    *)
        echo "❌ Unknown build type: $1"
        echo ""
        show_help
        exit 1
        ;;
esac