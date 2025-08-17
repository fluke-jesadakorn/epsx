#!/bin/bash

# EPSX Container Engine Detection and Performance Check
# Helps verify OrbStack migration and provides performance insights

set -e

echo "🔍 EPSX Container Engine Analysis"
echo "================================="
echo ""

# Detect container engine
detect_engine() {
    if docker info 2>/dev/null | grep -q "OrbStack"; then
        echo "orbstack"
    elif docker info 2>/dev/null | grep -q "Docker Desktop"; then
        echo "docker-desktop"
    elif command -v podman &> /dev/null && podman info 2>/dev/null >/dev/null; then
        echo "podman"
    else
        echo "unknown"
    fi
}

# Performance test function
performance_test() {
    local engine=$1
    echo "⚡ Running performance test..."
    
    # Container startup test
    echo "   📦 Testing container startup speed..."
    start_time=$(date +%s%N)
    docker run --rm alpine:latest echo "Hello from $engine" >/dev/null 2>&1
    end_time=$(date +%s%N)
    
    startup_ms=$(( (end_time - start_time) / 1000000 ))
    startup_sec=$(( startup_ms / 1000 ))
    
    echo "   ⏱️  Container startup: ${startup_sec}s (${startup_ms}ms)"
    
    # Performance rating
    if [ "$startup_ms" -lt 3000 ]; then
        echo "   🌟 Excellent performance!"
    elif [ "$startup_ms" -lt 10000 ]; then
        echo "   ✅ Good performance"
    elif [ "$startup_ms" -lt 30000 ]; then
        echo "   ⚠️  Moderate performance"
    else
        echo "   🐌 Slow performance - consider OrbStack migration"
    fi
    
    return $startup_ms
}

# Check system resources
check_resources() {
    echo "💻 System Resource Usage:"
    
    # CPU usage for container processes
    container_cpu=$(ps aux | grep -E "(docker|orbstack|podman)" | grep -v grep | awk '{sum += $3} END {printf "%.1f", sum+0}')
    echo "   🖥️  Container CPU usage: ${container_cpu}%"
    
    # Memory usage
    if command -v docker &> /dev/null; then
        docker_mem=$(ps aux | grep -E "docker|orbstack" | grep -v grep | awk '{sum += $6} END {printf "%.0f", (sum+0)/1024}')
        echo "   💾 Container memory usage: ${docker_mem}MB"
    fi
    
    # Battery status (macOS only)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        battery_info=$(pmset -g batt | grep -o '[0-9]*%' | head -1)
        power_source=$(pmset -g batt | grep -o 'AC Power\|Battery Power' | head -1)
        echo "   🔋 Battery: $battery_info ($power_source)"
    fi
}

# Main detection and analysis
ENGINE=$(detect_engine)

echo "🖥️  Container Engine: $ENGINE"
echo "🏗️  Host Architecture: $(uname -m)"
echo "🎯 Target Platform: linux/amd64 (Cloud Run)"
echo ""

# Engine-specific information
case $ENGINE in
    "orbstack")
        echo "🌟 OrbStack Detected - Excellent Choice!"
        echo "   ⚡ Sub-second container startup"
        echo "   💚 Native ARM64 optimization"
        echo "   🔋 4x better battery efficiency"
        echo "   💾 Minimal resource footprint"
        echo ""
        
        # OrbStack version info
        if command -v orb &> /dev/null; then
            orb_version=$(orb --version 2>/dev/null || echo "unknown")
            echo "   📦 OrbStack version: $orb_version"
        fi
        ;;
        
    "docker-desktop")
        echo "⚠️  Docker Desktop Detected"
        echo "   🐌 Slower startup times (20-30s typical)"
        echo "   🔋 Higher battery usage"
        echo "   💾 Larger memory footprint"
        echo ""
        echo "💡 Recommendation: Migrate to OrbStack"
        echo "   📈 Expected improvements:"
        echo "   • 15x faster container startup"
        echo "   • 75% lower battery usage"
        echo "   • 3-4x faster builds"
        echo "   • Better system responsiveness"
        echo ""
        ;;
        
    "podman")
        echo "🔧 Podman Detected - Enterprise Grade"
        echo "   🔒 Daemonless architecture"
        echo "   🏢 Rootless containers"
        echo "   💰 Cost-effective solution"
        echo ""
        
        # Podman version info
        podman_version=$(podman --version 2>/dev/null || echo "unknown")
        echo "   📦 Podman version: $podman_version"
        ;;
        
    "unknown")
        echo "❌ No Container Engine Detected"
        echo "   Please install one of the following:"
        echo "   • OrbStack (recommended for macOS): https://orbstack.dev"
        echo "   • Docker Desktop: https://docker.com/desktop"
        echo "   • Podman: https://podman.io"
        echo ""
        exit 1
        ;;
esac

# Performance testing
if [ "$ENGINE" != "unknown" ]; then
    echo "🧪 Performance Analysis:"
    performance_test "$ENGINE"
    echo ""
fi

# Resource usage check
check_resources
echo ""

# EPSX-specific checks
echo "🏗️  EPSX Build Environment Check:"

# Check if buildx is available
if docker buildx inspect >/dev/null 2>&1; then
    echo "   ✅ Docker buildx available"
    
    # Check supported platforms
    platforms=$(docker buildx inspect | grep "Platforms:" | cut -d: -f2 | tr -d ' ')
    if echo "$platforms" | grep -q "linux/amd64"; then
        echo "   ✅ AMD64 platform support (Cloud Run compatible)"
    else
        echo "   ⚠️  AMD64 platform support missing"
    fi
    
    if echo "$platforms" | grep -q "linux/arm64"; then
        echo "   ✅ ARM64 platform support (native builds)"
    fi
else
    echo "   ❌ Docker buildx not available"
fi

# Check build cache
cache_dir="/tmp/.buildx-cache-orbstack"
if [ -d "$cache_dir" ]; then
    cache_size=$(du -sh "$cache_dir" 2>/dev/null | cut -f1)
    echo "   📦 Build cache: $cache_size ($cache_dir)"
else
    echo "   📦 Build cache: Not initialized"
fi

echo ""
echo "📊 Optimization Score:"

# Calculate optimization score
score=0

case $ENGINE in
    "orbstack") score=$((score + 40)) ;;
    "podman") score=$((score + 30)) ;;
    "docker-desktop") score=$((score + 10)) ;;
esac

if docker buildx inspect >/dev/null 2>&1; then
    score=$((score + 20))
fi

if echo "$platforms" | grep -q "linux/amd64"; then
    score=$((score + 20))
fi

if echo "$platforms" | grep -q "linux/arm64"; then
    score=$((score + 10))
fi

if [ -d "$cache_dir" ]; then
    score=$((score + 10))
fi

echo "   🎯 Total Score: $score/100"

if [ $score -ge 80 ]; then
    echo "   🌟 Excellent - Optimized for high performance!"
elif [ $score -ge 60 ]; then
    echo "   ✅ Good - Well configured for development"
elif [ $score -ge 40 ]; then
    echo "   ⚠️  Fair - Some optimizations possible"
else
    echo "   🚨 Poor - Significant improvements needed"
fi

echo ""
echo "⏭️  Next Steps:"

if [ "$ENGINE" = "docker-desktop" ]; then
    echo "   1. 📋 Review migration guide: ./scripts/orbstack-migration-guide.md"
    echo "   2. 🚀 Install OrbStack: https://orbstack.dev"
    echo "   3. ⚡ Experience 15x performance improvement"
elif [ "$ENGINE" = "orbstack" ]; then
    echo "   1. 🏗️  Run optimized build: ./scripts/build.sh"
    echo "   2. 📊 Monitor performance improvements"
    echo "   3. 🌟 Enjoy faster development workflow"
elif [ "$ENGINE" = "podman" ]; then
    echo "   1. 🔧 Verify podman configuration"
    echo "   2. 🏗️  Test EPSX build process"
    echo "   3. 📚 Review enterprise features"
fi

echo ""
echo "🔧 For issues or questions:"
echo "   • Check: ./scripts/orbstack-migration-guide.md"
echo "   • Team Slack: #development channel"
echo "   • OrbStack docs: https://orbstack.dev/docs"

echo ""
echo "🚀 EPSX container environment analysis complete!"