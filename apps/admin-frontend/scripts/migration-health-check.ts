#!/usr/bin/env tsx
/**
 * Migration Health Check Script
 * Validates that the unified user management system is working correctly
 * and helps identify issues during the migration from legacy routes
 */

import { existsSync, readFileSync } from 'fs'
import { resolve } from 'path'

interface HealthCheckResult {
  category: string
  check: string
  status: 'pass' | 'fail' | 'warning'
  message: string
  suggestion?: string
}

class MigrationHealthChecker {
  private results: HealthCheckResult[] = []
  private appPath: string

  constructor() {
    this.appPath = resolve(__dirname, '..')
  }

  private addResult(category: string, check: string, status: 'pass' | 'fail' | 'warning', message: string, suggestion?: string) {
    this.results.push({ category, check, status, message, suggestion })
  }

  /**
   * Check if all required files exist
   */
  checkFileStructure() {
    const requiredFiles = [
      // Core routing files
      'app/users/page.tsx',
      'app/users/[userId]/page.tsx',
      'app/users/[userId]/permissions/page.tsx',
      'app/users/[userId]/modules/page.tsx',
      'app/users/[userId]/packages/page.tsx',
      'app/users/[userId]/activity/page.tsx',
      
      // Core components
      'components/users/EnhancedUserList.tsx',
      'components/users/UserCard.tsx',
      'components/users/UserListFilters.tsx',
      'components/users/UserListPagination.tsx',
      'components/users/UserListSkeleton.tsx',
      
      // Tab components
      'components/users/UserPermissionsContent.tsx',
      'components/users/UserModulesContent.tsx',
      'components/users/UserPackagesContent.tsx',
      'components/users/UserActivityContent.tsx',
      
      // Server actions
      'lib/actions/unified-user-actions.ts',
      'lib/actions/user-list-actions.ts',
      
      // Types
      'lib/types/unified-user.ts',
      
      // Middleware
      'middleware.ts',
      
      // Compatibility layer
      'lib/compat/legacy-api-adapter.ts'
    ]

    const category = 'File Structure'
    
    requiredFiles.forEach(file => {
      const fullPath = resolve(this.appPath, file)
      if (existsSync(fullPath)) {
        this.addResult(category, file, 'pass', 'File exists')
      } else {
        this.addResult(category, file, 'fail', 'Required file missing', 
          'This file is required for the unified user management system')
      }
    })
  }

  /**
   * Check middleware configuration
   */
  checkMiddlewareConfig() {
    const category = 'Middleware Configuration'
    const middlewarePath = resolve(this.appPath, 'middleware.ts')
    
    if (!existsSync(middlewarePath)) {
      this.addResult(category, 'middleware.ts', 'fail', 'Middleware file missing')
      return
    }

    const middlewareContent = readFileSync(middlewarePath, 'utf8')
    
    // Check that legacy routes are removed (clean migration approach)
    const legacyRoutes = ['/iam', '/modules', '/billing', '/stock-ranking-packages']
    const hasLegacyRoutes = legacyRoutes.some(route => 
      middlewareContent.includes(`'${route}'`) || middlewareContent.includes(`"${route}"`)
    )

    if (!hasLegacyRoutes) {
      this.addResult(category, 'Legacy Routes Cleanup', 'pass', 'Legacy routes have been properly removed')
    } else {
      this.addResult(category, 'Legacy Routes Cleanup', 'warning', 'Legacy route references still found',
        'Remove legacy route references for clean migration')
    }

    // Check that redirect logic is removed (clean migration)
    if (!middlewareContent.includes('301') && !middlewareContent.includes('redirect')) {
      this.addResult(category, 'Clean Migration', 'pass', 'Redirect logic properly removed')
    } else {
      this.addResult(category, 'Clean Migration', 'warning', 'Redirect references still found',
        'Remove redirect logic for clean migration approach')
    }
  }

  /**
   * Check feature flag configuration
   */
  checkFeatureFlags() {
    const category = 'Feature Flags'
    const envPath = resolve(this.appPath, '.env.local')
    const envExamplePath = resolve(this.appPath, '.env.example')
    
    let envContent = ''
    
    if (existsSync(envPath)) {
      envContent = readFileSync(envPath, 'utf8')
      this.addResult(category, 'Environment File', 'pass', '.env.local found')
    } else if (existsSync(envExamplePath)) {
      envContent = readFileSync(envExamplePath, 'utf8')
      this.addResult(category, 'Environment File', 'warning', 'Using .env.example, create .env.local')
    } else {
      this.addResult(category, 'Environment File', 'fail', 'No environment file found')
      return
    }

    if (envContent.includes('UNIFIED_USER_MANAGEMENT')) {
      this.addResult(category, 'Unified User Management Flag', 'pass', 'Feature flag configured')
    } else {
      this.addResult(category, 'Unified User Management Flag', 'warning', 'Feature flag not found',
        'Add UNIFIED_USER_MANAGEMENT=true to enable the new system')
    }
  }

  /**
   * Check package.json for required dependencies
   */
  checkDependencies() {
    const category = 'Dependencies'
    const packagePath = resolve(this.appPath, 'package.json')
    
    if (!existsSync(packagePath)) {
      this.addResult(category, 'package.json', 'fail', 'package.json not found')
      return
    }

    const packageContent = JSON.parse(readFileSync(packagePath, 'utf8'))
    const dependencies = { ...packageContent.dependencies, ...packageContent.devDependencies }

    const requiredDeps = [
      { name: 'next', version: '15', type: 'framework' },
      { name: 'react', version: '19', type: 'framework' },
      { name: 'lucide-react', version: null, type: 'icons' },
      { name: '@playwright/test', version: null, type: 'testing' }
    ]

    requiredDeps.forEach(dep => {
      if (dependencies[dep.name]) {
        const version = dependencies[dep.name].replace(/[\^~]/, '')
        if (dep.version && !version.startsWith(dep.version)) {
          this.addResult(category, dep.name, 'warning', 
            `Version ${version} found, ${dep.version}+ recommended`)
        } else {
          this.addResult(category, dep.name, 'pass', `${dep.type} dependency found`)
        }
      } else {
        this.addResult(category, dep.name, 'fail', `Required ${dep.type} dependency missing`)
      }
    })
  }

  /**
   * Check TypeScript configuration
   */
  checkTypeScriptConfig() {
    const category = 'TypeScript Configuration'
    const tsconfigPath = resolve(this.appPath, 'tsconfig.json')
    
    if (!existsSync(tsconfigPath)) {
      this.addResult(category, 'tsconfig.json', 'fail', 'TypeScript config not found')
      return
    }

    const tsconfigContent = JSON.parse(readFileSync(tsconfigPath, 'utf8'))
    
    // Check for strict mode
    if (tsconfigContent.compilerOptions?.strict) {
      this.addResult(category, 'Strict Mode', 'pass', 'TypeScript strict mode enabled')
    } else {
      this.addResult(category, 'Strict Mode', 'warning', 'TypeScript strict mode disabled',
        'Enable strict mode for better type safety')
    }

    // Check for path mapping
    if (tsconfigContent.compilerOptions?.paths) {
      this.addResult(category, 'Path Mapping', 'pass', 'Path mapping configured')
    } else {
      this.addResult(category, 'Path Mapping', 'warning', 'Path mapping not configured',
        'Configure path mapping for cleaner imports')
    }
  }

  /**
   * Check test configuration
   */
  checkTestConfiguration() {
    const category = 'Test Configuration'
    
    const testFiles = [
      '__tests__/integration/unified-user-management.test.ts',
      'e2e/user-profile-tabs.spec.ts',
      'jest.config.js',
      'playwright.config.ts'
    ]

    testFiles.forEach(file => {
      const fullPath = resolve(this.appPath, file)
      if (existsSync(fullPath)) {
        this.addResult(category, file, 'pass', 'Test file exists')
      } else {
        this.addResult(category, file, 'warning', 'Test file missing',
          'Add this test file to ensure system reliability')
      }
    })
  }

  /**
   * Run all health checks
   */
  async runAllChecks() {
    console.log('🔍 Running Migration Health Checks...\n')
    
    this.checkFileStructure()
    this.checkMiddlewareConfig()
    this.checkFeatureFlags()
    this.checkDependencies()
    this.checkTypeScriptConfig()
    this.checkTestConfiguration()
    
    this.printResults()
    this.printSummary()
    
    const hasFailures = this.results.some(r => r.status === 'fail')
    process.exit(hasFailures ? 1 : 0)
  }

  /**
   * Print detailed results
   */
  private printResults() {
    const categories = [...new Set(this.results.map(r => r.category))]
    
    categories.forEach(category => {
      console.log(`\n📁 ${category}`)
      console.log('─'.repeat(50))
      
      const categoryResults = this.results.filter(r => r.category === category)
      categoryResults.forEach(result => {
        const icon = result.status === 'pass' ? '✅' : result.status === 'warning' ? '⚠️' : '❌'
        console.log(`${icon} ${result.check}: ${result.message}`)
        
        if (result.suggestion) {
          console.log(`   💡 ${result.suggestion}`)
        }
      })
    })
  }

  /**
   * Print summary statistics
   */
  private printSummary() {
    const passed = this.results.filter(r => r.status === 'pass').length
    const warnings = this.results.filter(r => r.status === 'warning').length
    const failed = this.results.filter(r => r.status === 'fail').length
    const total = this.results.length

    console.log('\n📊 Summary')
    console.log('─'.repeat(50))
    console.log(`Total Checks: ${total}`)
    console.log(`✅ Passed: ${passed}`)
    console.log(`⚠️  Warnings: ${warnings}`)
    console.log(`❌ Failed: ${failed}`)
    
    const score = Math.round((passed / total) * 100)
    console.log(`\n🎯 Health Score: ${score}%`)
    
    if (score >= 90) {
      console.log('🎉 Excellent! Your migration is ready for production.')
    } else if (score >= 75) {
      console.log('👍 Good! Address warnings to improve reliability.')
    } else if (score >= 50) {
      console.log('⚡ Fair. Several issues need attention before deployment.')
    } else {
      console.log('🚨 Critical issues detected. Migration is not ready.')
    }
  }
}

// Run the health check if this script is executed directly
if (require.main === module) {
  const checker = new MigrationHealthChecker()
  checker.runAllChecks().catch(error => {
    console.error('❌ Health check failed:', error)
    process.exit(1)
  })
}

export { MigrationHealthChecker }