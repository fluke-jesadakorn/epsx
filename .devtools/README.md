# EPSX Development Tools

This directory contains development and debugging tools for the EPSX platform. These tools are designed to help developers troubleshoot issues, monitor performance, and debug the application during development.

## 🔧 Available Tools

### Environment Management & Debugging

#### `debug-env.sh` - Environment Variable Debugger
Comprehensive environment variable inspection and debugging tool.

```bash
# Show environment summary
./.devtools/debug-env.sh

# Show all environment variables
./.devtools/debug-env.sh --all

# Check for missing required variables
./.devtools/debug-env.sh --missing

# Show environment files status
./.devtools/debug-env.sh --files

# Export current environment
./.devtools/debug-env.sh --export
```

**Features:**
- ✅ Environment variable inspection
- ✅ Missing variable detection
- ✅ Environment file status checking
- ✅ Environment export functionality
- ✅ Secure secret masking

#### `compare-envs.sh` - Environment Comparison Tool (Planned)
Compare different environment configurations side-by-side.

```bash
# Compare development vs production
./.devtools/compare-envs.sh development production

# Compare current vs expected environment
./.devtools/compare-envs.sh --current --expected=staging
```

### Build & Performance

#### `troubleshoot-build.sh` - Build Issue Diagnosis (Planned)
Diagnose and troubleshoot build issues across the monorepo.

```bash
# Diagnose general build issues
./.devtools/troubleshoot-build.sh

# Check specific app build
./.devtools/troubleshoot-build.sh --app=frontend

# Check dependencies
./.devtools/troubleshoot-build.sh --deps
```

#### `performance-monitor.sh` - Performance Monitoring (Planned)
Monitor build and runtime performance metrics.

```bash
# Monitor build performance
./.devtools/performance-monitor.sh build

# Monitor runtime performance
./.devtools/performance-monitor.sh runtime --env=production
```

### WebSocket & Trading Data

#### `tradingview_ws_debug/` - TradingView WebSocket Debugging (Existing)
Contains WebSocket debugging data and logs for TradingView integration.

**Current files:**
- `MSFT_websocket_debug_2025-08-10T15-15-58-055Z.json` - WebSocket debug data

### Container & Deployment

#### `container-health.sh` - Container Health Checker (Planned)
Check the health and status of Docker containers.

```bash
# Check all containers
./.devtools/container-health.sh

# Check specific service
./.devtools/container-health.sh --service=backend
```

#### `docker-debug.sh` - Docker Debugging Tools (Planned)
Tools for debugging Docker containers and images.

```bash
# Debug container issues
./.devtools/docker-debug.sh --container=frontend

# Analyze image sizes
./.devtools/docker-debug.sh --image-size
```

### Logging & Analysis

#### `logs-analyzer.sh` - Log Analysis Tool (Planned)
Analyze application logs across different services and environments.

```bash
# Analyze recent logs
./.devtools/logs-analyzer.sh --recent

# Analyze errors
./.devtools/logs-analyzer.sh --errors --service=backend
```

### Data & Backup

#### `env-backup.sh` - Environment Backup Tool (Planned)
Backup and restore environment configurations.

```bash
# Backup current environment
./.devtools/env-backup.sh --backup

# Restore environment
./.devtools/env-backup.sh --restore=backup-20250110-120000
```

#### `dependency-analyzer.sh` - Dependency Analysis (Planned)
Analyze project dependencies and check for issues.

```bash
# Analyze all dependencies
./.devtools/dependency-analyzer.sh

# Check for outdated packages
./.devtools/dependency-analyzer.sh --outdated
```

### Workspace Management

#### `workspace-health.sh` - Workspace Health Check (Planned)
Check the overall health of the monorepo workspace.

```bash
# Full workspace health check
./.devtools/workspace-health.sh

# Quick health check
./.devtools/workspace-health.sh --quick
```

## 🚀 Quick Start

1. **Environment Debugging**: Start with environment debugging to ensure your setup is correct:
   ```bash
   ./.devtools/debug-env.sh --missing
   ```

2. **Check Environment Files**: Verify all environment files exist:
   ```bash
   ./.devtools/debug-env.sh --files
   ```

3. **Export Current Environment**: Save current configuration:
   ```bash
   ./.devtools/debug-env.sh --export
   ```

## 📁 Directory Structure

```
.devtools/
├── README.md                      # This documentation
├── debug-env.sh                   # Environment debugging (✅ Available)
├── compare-envs.sh                # Environment comparison (Planned)
├── troubleshoot-build.sh          # Build troubleshooting (Planned)
├── performance-monitor.sh         # Performance monitoring (Planned)
├── container-health.sh            # Container health checks (Planned)
├── docker-debug.sh                # Docker debugging (Planned)
├── logs-analyzer.sh               # Log analysis (Planned)
├── env-backup.sh                  # Environment backup (Planned)
├── dependency-analyzer.sh         # Dependency analysis (Planned)
├── workspace-health.sh            # Workspace health (Planned)
├── secret-audit.sh                # Secret scanning (Planned)
└── tradingview_ws_debug/          # WebSocket debugging (✅ Available)
    └── MSFT_websocket_debug_...   # Debug data files
```

## 🔒 Security Notes

- **Secret Detection**: The debug tools automatically mask sensitive information
- **Log Safety**: Debug outputs are designed to be safe for sharing
- **File Permissions**: All scripts require explicit execution permissions
- **Data Isolation**: Debug data is stored separately from production code

## 🛠️ Development Workflow Integration

These tools integrate with the main development workflow:

```bash
# 1. Setup environment
pnpm env:dev

# 2. Debug environment
./.devtools/debug-env.sh --missing

# 3. Build for development
pnpm build:dev

# 4. Troubleshoot if needed
./.devtools/troubleshoot-build.sh

# 5. Deploy
pnpm deploy:staging
```

## 📊 Tool Status

| Tool | Status | Description |
|------|--------|-------------|
| `debug-env.sh` | ✅ Available | Environment debugging and inspection |
| `tradingview_ws_debug/` | ✅ Available | WebSocket debugging data |
| `compare-envs.sh` | 🔄 Planned | Environment comparison |
| `troubleshoot-build.sh` | 🔄 Planned | Build issue diagnosis |
| `performance-monitor.sh` | 🔄 Planned | Performance monitoring |
| Other tools | 🔄 Planned | Various debugging utilities |

## 🤝 Contributing

When adding new debugging tools:

1. Follow the naming convention: `kebab-case.sh`
2. Add executable permissions: `chmod +x script.sh`
3. Include help/usage function
4. Update this README
5. Add to the package.json scripts if appropriate

## 📝 Examples

### Common Debugging Scenarios

**Environment not loading correctly:**
```bash
./.devtools/debug-env.sh --files
./.devtools/debug-env.sh --missing
```

**Build failing:**
```bash
./.devtools/troubleshoot-build.sh --app=frontend
```

**Performance issues:**
```bash
./.devtools/performance-monitor.sh build --verbose
```

**Container problems:**
```bash
./.devtools/container-health.sh --service=backend --verbose
```