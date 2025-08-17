#!/bin/bash

# EPSX Performance Analysis Script
# Analyzes benchmark data to provide performance insights and trends

set -e

BENCHMARK_DATA_DIR="./logs/benchmarks"
BENCHMARK_LOG_FILE="./logs/build-performance.log"

echo "📊 EPSX Performance Analysis"
echo "============================"
echo ""

# Check if benchmark data exists
if [ ! -d "$BENCHMARK_DATA_DIR" ] || [ ! -f "$BENCHMARK_LOG_FILE" ]; then
    echo "❌ No benchmark data found."
    echo "   Run './scripts/benchmark-builds.sh' first to collect performance data."
    exit 1
fi

# Analyze recent performance trends
analyze_trends() {
    echo "📈 Performance Trends (Last 7 Days):"
    
    local recent_data=$(grep "$(date -v-7d '+%Y-%m-%d' 2>/dev/null || date -d '7 days ago' '+%Y-%m-%d' 2>/dev/null || echo '1970-01-01')" -A 1000 "$BENCHMARK_LOG_FILE" 2>/dev/null || echo "")
    
    if [ -z "$recent_data" ]; then
        echo "   📊 No recent data available"
        return
    fi
    
    # Extract performance data by engine
    local orbstack_data=$(echo "$recent_data" | grep "orbstack" || echo "")
    local docker_data=$(echo "$recent_data" | grep "docker-desktop" || echo "")
    local podman_data=$(echo "$recent_data" | grep "podman" || echo "")
    
    if [ -n "$orbstack_data" ]; then
        local orbstack_count=$(echo "$orbstack_data" | wc -l | xargs)
        local avg_startup=$(echo "$orbstack_data" | grep -o 'Startup: [0-9]*ms' | grep -o '[0-9]*' | awk '{sum+=$1} END {printf "%.0f", sum/NR}')
        local avg_build=$(echo "$orbstack_data" | grep -o 'Build: [0-9]*s' | grep -o '[0-9]*' | awk '{sum+=$1} END {printf "%.0f", sum/NR}')
        
        echo "   🌟 OrbStack ($orbstack_count measurements):"
        echo "      Average startup: ${avg_startup}ms"
        echo "      Average build: ${avg_build}s"
    fi
    
    if [ -n "$docker_data" ]; then
        local docker_count=$(echo "$docker_data" | wc -l | xargs)
        local avg_startup=$(echo "$docker_data" | grep -o 'Startup: [0-9]*ms' | grep -o '[0-9]*' | awk '{sum+=$1} END {printf "%.0f", sum/NR}')
        local avg_build=$(echo "$docker_data" | grep -o 'Build: [0-9]*s' | grep -o '[0-9]*' | awk '{sum+=$1} END {printf "%.0f", sum/NR}')
        
        echo "   🐳 Docker Desktop ($docker_count measurements):"
        echo "      Average startup: ${avg_startup}ms"
        echo "      Average build: ${avg_build}s"
    fi
    
    if [ -n "$podman_data" ]; then
        local podman_count=$(echo "$podman_data" | wc -l | xargs)
        local avg_startup=$(echo "$podman_data" | grep -o 'Startup: [0-9]*ms' | grep -o '[0-9]*' | awk '{sum+=$1} END {printf "%.0f", sum/NR}')
        local avg_build=$(echo "$podman_data" | grep -o 'Build: [0-9]*s' | grep -o '[0-9]*' | awk '{sum+=$1} END {printf "%.0f", sum/NR}')
        
        echo "   🔧 Podman ($podman_count measurements):"
        echo "      Average startup: ${avg_startup}ms"
        echo "      Average build: ${avg_build}s"
    fi
    
    echo ""
}

# Compare engines
compare_engines() {
    echo "⚔️  Engine Performance Comparison:"
    
    # Get latest measurement for each engine
    local orbstack_latest=$(grep "orbstack" "$BENCHMARK_LOG_FILE" | tail -1 || echo "")
    local docker_latest=$(grep "docker-desktop" "$BENCHMARK_LOG_FILE" | tail -1 || echo "")
    local podman_latest=$(grep "podman" "$BENCHMARK_LOG_FILE" | tail -1 || echo "")
    
    if [ -n "$orbstack_latest" ] && [ -n "$docker_latest" ]; then
        local orbstack_startup=$(echo "$orbstack_latest" | grep -o 'Startup: [0-9]*ms' | grep -o '[0-9]*')
        local orbstack_build=$(echo "$orbstack_latest" | grep -o 'Build: [0-9]*s' | grep -o '[0-9]*')
        local docker_startup=$(echo "$docker_latest" | grep -o 'Startup: [0-9]*ms' | grep -o '[0-9]*')
        local docker_build=$(echo "$docker_latest" | grep -o 'Build: [0-9]*s' | grep -o '[0-9]*')
        
        local startup_improvement=$(echo "scale=1; $docker_startup / $orbstack_startup" | bc 2>/dev/null || echo "N/A")
        local build_improvement=$(echo "scale=1; $docker_build / $orbstack_build" | bc 2>/dev/null || echo "N/A")
        
        echo "   🌟 OrbStack vs Docker Desktop:"
        echo "      Startup improvement: ${startup_improvement}x faster"
        echo "      Build improvement: ${build_improvement}x faster"
        echo ""
    fi
    
    # Show current best performer
    local best_startup=999999
    local best_build=999999
    local best_engine=""
    
    for engine_data in "$orbstack_latest" "$docker_latest" "$podman_latest"; do
        if [ -n "$engine_data" ]; then
            local startup=$(echo "$engine_data" | grep -o 'Startup: [0-9]*ms' | grep -o '[0-9]*')
            local build=$(echo "$engine_data" | grep -o 'Build: [0-9]*s' | grep -o '[0-9]*')
            local engine=$(echo "$engine_data" | grep -o 'Engine: [a-z-]*' | cut -d: -f2 | xargs)
            
            if [ "$startup" -lt "$best_startup" ] && [ "$build" -lt "$best_build" ]; then
                best_startup="$startup"
                best_build="$build"
                best_engine="$engine"
            fi
        fi
    done
    
    if [ -n "$best_engine" ]; then
        echo "   🏆 Current Best Performer: $best_engine"
        echo "      Startup: ${best_startup}ms"
        echo "      Build: ${best_build}s"
        echo ""
    fi
}

# Migration impact analysis
migration_impact() {
    echo "🔄 Migration Impact Analysis:"
    
    # Check if there's data from both Docker Desktop and OrbStack
    local has_docker=$(grep "docker-desktop" "$BENCHMARK_LOG_FILE" >/dev/null && echo "true" || echo "false")
    local has_orbstack=$(grep "orbstack" "$BENCHMARK_LOG_FILE" >/dev/null && echo "true" || echo "false")
    
    if [ "$has_docker" = "true" ] && [ "$has_orbstack" = "true" ]; then
        local docker_avg_startup=$(grep "docker-desktop" "$BENCHMARK_LOG_FILE" | grep -o 'Startup: [0-9]*ms' | grep -o '[0-9]*' | awk '{sum+=$1; count++} END {printf "%.0f", sum/count}')
        local docker_avg_build=$(grep "docker-desktop" "$BENCHMARK_LOG_FILE" | grep -o 'Build: [0-9]*s' | grep -o '[0-9]*' | awk '{sum+=$1; count++} END {printf "%.0f", sum/count}')
        
        local orbstack_avg_startup=$(grep "orbstack" "$BENCHMARK_LOG_FILE" | grep -o 'Startup: [0-9]*ms' | grep -o '[0-9]*' | awk '{sum+=$1; count++} END {printf "%.0f", sum/count}')
        local orbstack_avg_build=$(grep "orbstack" "$BENCHMARK_LOG_FILE" | grep -o 'Build: [0-9]*s' | grep -o '[0-9]*' | awk '{sum+=$1; count++} END {printf "%.0f", sum/count}')
        
        local startup_saved=$((docker_avg_startup - orbstack_avg_startup))
        local build_saved=$((docker_avg_build - orbstack_avg_build))
        
        echo "   📊 OrbStack Migration Benefits:"
        echo "      Startup time saved: ${startup_saved}ms per container"
        echo "      Build time saved: ${build_saved}s per build"
        echo ""
        
        # Calculate daily time savings (assuming 10 container starts, 3 builds per day)
        local daily_startup_savings=$((startup_saved * 10 / 1000))  # Convert to seconds
        local daily_build_savings=$((build_saved * 3))
        local daily_total=$((daily_startup_savings + daily_build_savings))
        
        if [ $daily_total -gt 0 ]; then
            echo "   ⏰ Estimated Daily Time Savings:"
            echo "      Container startups: ${daily_startup_savings}s"
            echo "      Builds: ${daily_build_savings}s"
            echo "      Total: ${daily_total}s per developer per day"
            echo ""
            
            # Weekly and monthly projections
            local weekly_savings=$((daily_total * 5))
            local monthly_savings=$((daily_total * 22))  # 22 working days
            
            echo "   📅 Productivity Impact:"
            echo "      Weekly savings: ${weekly_savings}s ($(echo "scale=1; $weekly_savings/60" | bc | sed 's/^\./0./')min)"
            echo "      Monthly savings: ${monthly_savings}s ($(echo "scale=1; $monthly_savings/3600" | bc | sed 's/^\./0./')hr)"
        fi
    elif [ "$has_orbstack" = "true" ]; then
        echo "   🌟 Already using OrbStack - excellent choice!"
        echo "   📈 Continue monitoring performance for optimizations"
    elif [ "$has_docker" = "true" ]; then
        echo "   💡 Currently using Docker Desktop"
        echo "   🚀 Consider migrating to OrbStack for:"
        echo "      • 15x faster container startup"
        echo "      • 3-4x faster builds"
        echo "      • 75% battery life improvement"
        echo "      • Guide: ./scripts/orbstack-migration-guide.md"
    fi
    
    echo ""
}

# Performance recommendations
generate_recommendations() {
    echo "💡 Performance Recommendations:"
    
    # Get latest performance data
    local latest_entry=$(tail -1 "$BENCHMARK_LOG_FILE" || echo "")
    
    if [ -n "$latest_entry" ]; then
        local current_engine=$(echo "$latest_entry" | grep -o 'Engine: [a-z-]*' | cut -d: -f2 | xargs)
        local startup_time=$(echo "$latest_entry" | grep -o 'Startup: [0-9]*ms' | grep -o '[0-9]*')
        local build_time=$(echo "$latest_entry" | grep -o 'Build: [0-9]*s' | grep -o '[0-9]*')
        
        echo "   🖥️  Current Engine: $current_engine"
        echo ""
        
        # Engine-specific recommendations
        case $current_engine in
            "orbstack")
                if [ "$startup_time" -gt 5000 ] || [ "$build_time" -gt 600 ]; then
                    echo "   🔧 OrbStack Optimization Opportunities:"
                    [ "$startup_time" -gt 5000 ] && echo "      • Container startup slower than expected - check system resources"
                    [ "$build_time" -gt 600 ] && echo "      • Build time could be improved - review build cache configuration"
                    echo "      • Run system performance check: ./scripts/check-container-engine.sh"
                else
                    echo "   🌟 Excellent OrbStack performance!"
                    echo "      • Your setup is well-optimized"
                    echo "      • Continue current configuration"
                fi
                ;;
                
            "docker-desktop")
                echo "   🚀 High-Impact Optimization Available:"
                echo "      • Migrate to OrbStack for dramatic improvements"
                echo "      • Expected startup improvement: 15x faster"
                echo "      • Expected build improvement: 3-4x faster"
                echo "      • Guide: ./scripts/orbstack-migration-guide.md"
                echo "      • Download: https://orbstack.dev"
                ;;
                
            "podman")
                echo "   🔧 Podman Optimization Tips:"
                echo "      • Verify ARM64 machine configuration"
                echo "      • Consider OrbStack for development speed"
                echo "      • Leverage Podman's security for production"
                ;;
        esac
    fi
    
    echo ""
    echo "   📊 Monitoring Tips:"
    echo "      • Run benchmarks weekly: ./scripts/benchmark-builds.sh"
    echo "      • Compare clean vs incremental builds"
    echo "      • Monitor system resources during builds"
    echo "      • Track performance after system updates"
    
    echo ""
}

# Show benchmark history
show_history() {
    local days=${1:-7}
    echo "📜 Performance History (Last $days days):"
    
    local cutoff_date
    if [[ "$OSTYPE" == "darwin"* ]]; then
        cutoff_date=$(date -v-${days}d '+%Y-%m-%d' 2>/dev/null || echo '1970-01-01')
    else
        cutoff_date=$(date -d "$days days ago" '+%Y-%m-%d' 2>/dev/null || echo '1970-01-01')
    fi
    
    local recent_data=$(awk -v cutoff="$cutoff_date" '$0 >= "["cutoff' "$BENCHMARK_LOG_FILE" 2>/dev/null || echo "")
    
    if [ -z "$recent_data" ]; then
        echo "   📊 No data available for the last $days days"
        return
    fi
    
    echo "$recent_data" | while IFS= read -r line; do
        local date_time=$(echo "$line" | grep -o '\[.*\]' | tr -d '[]')
        local engine=$(echo "$line" | grep -o 'Engine: [a-z-]*' | cut -d: -f2 | xargs)
        local startup=$(echo "$line" | grep -o 'Startup: [0-9]*ms' | grep -o '[0-9]*')
        local build=$(echo "$line" | grep -o 'Build: [0-9]*s' | grep -o '[0-9]*')
        
        printf "   %s | %-15s | %4sms | %3ss\n" "$date_time" "$engine" "$startup" "$build"
    done
    
    echo ""
}

# Main analysis function
main() {
    local command=${1:-"summary"}
    
    case $command in
        "summary"|"")
            analyze_trends
            compare_engines
            migration_impact
            generate_recommendations
            ;;
        "history")
            show_history "${2:-7}"
            ;;
        "trends")
            analyze_trends
            ;;
        "compare")
            compare_engines
            ;;
        "recommendations")
            generate_recommendations
            ;;
        "migration")
            migration_impact
            ;;
        *)
            echo "Usage: $0 [command] [options]"
            echo ""
            echo "Commands:"
            echo "  summary          - Full performance analysis (default)"
            echo "  history [days]   - Show performance history"
            echo "  trends           - Show performance trends"
            echo "  compare          - Compare container engines"
            echo "  recommendations  - Get optimization recommendations"
            echo "  migration        - Analyze migration impact"
            echo ""
            echo "Examples:"
            echo "  $0                    # Full summary"
            echo "  $0 history           # Last 7 days history"
            echo "  $0 history 30        # Last 30 days history"
            echo "  $0 recommendations   # Get optimization tips"
            ;;
    esac
}

# Run main function with arguments
main "$@"