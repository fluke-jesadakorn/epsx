#!/usr/bin/env node

/**
 * Deployment Safety Script
 * Runs comprehensive checks before deployment
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

class DeploymentSafety {
  constructor() {
    this.projectRoot = process.cwd();
    this.checks = [];
    this.errors = [];
    this.warnings = [];
  }

  async runAll() {
    console.log('🔍 Running deployment safety checks...\n');
    
    const checks = [
      () => this.checkBuild(),
      () => this.checkTests(),
      () => this.checkLinting(),
      () => this.checkTypeCheck(),
      () => this.checkPerformance(),
      () => this.checkSecurity(),
      () => this.checkFeatureFlags(),
      () => this.checkMonitoring(),
      () => this.checkRollbackPlan(),
    ];

    for (const check of checks) {
      try {
        await check();
      } catch (error) {
        this.errors.push(error.message);
      }
    }

    return this.generateReport();
  }

  checkBuild() {
    console.log('📦 Checking build...');
    try {
      execSync('npm run build', { stdio: 'pipe' });
      this.logSuccess('Build completed successfully');
    } catch (error) {
      throw new Error(`Build failed: ${error.message}`);
    }
  }

  checkTests() {
    console.log('🧪 Running tests...');
    try {
      execSync('npm test', { stdio: 'pipe' });
      this.logSuccess('All tests passed');
    } catch (error) {
      throw new Error(`Tests failed: ${error.message}`);
    }
  }

  checkLinting() {
    console.log('🧹 Checking code quality...');
    try {
      execSync('npm run lint', { stdio: 'pipe' });
      this.logSuccess('Linting passed');
    } catch (error) {
      this.warnings.push(`Linting issues found: ${error.message}`);
      this.logWarning('Linting issues found (non-blocking)');
    }
  }

  checkTypeCheck() {
    console.log('🔍 Type checking...');
    try {
      execSync('npm run type-check', { stdio: 'pipe' });
      this.logSuccess('Type checking passed');
    } catch (error) {
      throw new Error(`Type checking failed: ${error.message}`);
    }
  }

  checkPerformance() {
    console.log('⚡ Validating performance targets...');
    try {
      const reportPath = path.join(this.projectRoot, 'reports/performance-validation.json');
      if (fs.existsSync(reportPath)) {
        const report = JSON.parse(fs.readFileSync(reportPath, 'utf8'));
        if (report.targetsAchieved) {
          this.logSuccess('Performance targets achieved');
        } else {
          throw new Error('Performance targets not met');
        }
      } else {
        this.warnings.push('Performance validation report not found');
        this.logWarning('Performance validation report not found');
      }
    } catch (error) {
      throw new Error(`Performance validation failed: ${error.message}`);
    }
  }

  checkSecurity() {
    console.log('🔒 Security checks...');
    try {
      // Check for common security issues
      this.checkEnvVars();
      this.checkSecretFiles();
      this.checkDependencies();
      this.logSuccess('Security checks passed');
    } catch (error) {
      throw new Error(`Security check failed: ${error.message}`);
    }
  }

  checkEnvVars() {
    const requiredEnvVars = [
      'NODE_ENV',
      'NEXT_PUBLIC_APP_ENV',
    ];

    const missingVars = requiredEnvVars.filter(varName => !process.env[varName]);
    if (missingVars.length > 0) {
      throw new Error(`Missing required environment variables: ${missingVars.join(', ')}`);
    }
  }

  checkSecretFiles() {
    const secretPatterns = [
      '.env.local',
      '.env.production.local',
      'credentials.json',
      'private-key.pem',
    ];

    secretPatterns.forEach(pattern => {
      if (fs.existsSync(path.join(this.projectRoot, pattern))) {
        this.warnings.push(`Secret file found: ${pattern}`);
      }
    });
  }

  checkDependencies() {
    console.log('  📋 Checking dependencies...');
    try {
      // Check for known vulnerabilities
      execSync('npm audit --audit-level moderate', { stdio: 'pipe' });
    } catch (error) {
      this.warnings.push('Dependency vulnerabilities found');
    }
  }

  checkFeatureFlags() {
    console.log('🚩 Validating feature flags...');
    try {
      const flagsPath = path.join(this.projectRoot, 'lib/feature-flags.ts');
      if (fs.existsSync(flagsPath)) {
        const flagsContent = fs.readFileSync(flagsPath, 'utf8');
        
        // Check if gradual rollout is enabled
        if (flagsContent.includes(\"'gradual-rollout'\")) {
          this.logSuccess('Feature flags system ready');
        } else {
          throw new Error('Gradual rollout feature flag not configured');
        }
      } else {
        throw new Error('Feature flags system not found');
      }
    } catch (error) {
      throw new Error(`Feature flags validation failed: ${error.message}`);
    }
  }

  checkMonitoring() {
    console.log('📊 Validating monitoring setup...');
    try {
      const monitoringPath = path.join(this.projectRoot, 'lib/monitoring.ts');
      const healthEndpoint = path.join(this.projectRoot, 'app/api/monitoring/health/route.ts');
      
      if (fs.existsSync(monitoringPath) && fs.existsSync(healthEndpoint)) {
        this.logSuccess('Monitoring system configured');
      } else {
        throw new Error('Monitoring system not properly configured');
      }
    } catch (error) {
      throw new Error(`Monitoring validation failed: ${error.message}`);
    }
  }

  checkRollbackPlan() {
    console.log('🔄 Validating rollback procedures...');
    try {
      const rollbackPath = path.join(this.projectRoot, 'lib/rollback.ts');
      if (fs.existsSync(rollbackPath)) {
        this.logSuccess('Rollback procedures configured');
      } else {
        throw new Error('Rollback procedures not configured');
      }
    } catch (error) {
      throw new Error(`Rollback validation failed: ${error.message}`);
    }
  }

  logSuccess(message) {
    console.log(`  ✅ ${message}`);
  }

  logWarning(message) {
    console.log(`  ⚠️  ${message}`);
  }

  logError(message) {
    console.log(`  ❌ ${message}`);
  }

  generateReport() {
    console.log('\n📋 Deployment Safety Report');
    console.log('================================');

    if (this.errors.length === 0) {
      console.log('✅ All critical checks passed - SAFE TO DEPLOY');
    } else {
      console.log('❌ Critical issues found - DO NOT DEPLOY');
      this.errors.forEach(error => this.logError(error));
    }

    if (this.warnings.length > 0) {
      console.log('\n⚠️  Warnings (non-blocking):');
      this.warnings.forEach(warning => console.log(`   - ${warning}`));
    }

    const report = {
      timestamp: new Date().toISOString(),
      status: this.errors.length === 0 ? 'SAFE' : 'UNSAFE',
      errors: this.errors,
      warnings: this.warnings,
      deployment_ready: this.errors.length === 0,
    };

    // Save report
    const reportPath = path.join(this.projectRoot, 'reports/deployment-safety.json');
    fs.mkdirSync(path.dirname(reportPath), { recursive: true });
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

    console.log(`\n📄 Report saved: ${reportPath}`);
    
    return report.deployment_ready;
  }
}

// CLI interface
async function main() {
  const safety = new DeploymentSafety();
  const isReady = await safety.runAll();
  
  process.exit(isReady ? 0 : 1);
}

if (require.main === module) {
  main().catch(error => {
    console.error('Deployment safety check failed:', error);
    process.exit(1);
  });
}

module.exports = DeploymentSafety;