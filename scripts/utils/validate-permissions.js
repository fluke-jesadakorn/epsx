#!/usr/bin/env node

// Permission Format Style Validator
// Ensures all permissions across the codebase follow unified format: "domain:action:scope"

const fs = require('fs');
const path = require('path');
const glob = require('glob');

// Unified permission format regex
const UNIFIED_FORMAT_REGEX = /^[a-z-]+:[a-z]+:[a-z]+$/;

// Valid enum values (matching backend Rust implementation)
const VALID_DOMAINS = [
  'users', 'roles', 'permissions', 'analytics', 'packages', 'payments', 'dashboard',
  'admin-users', 'admin-roles', 'admin-security', 'admin-system', 'admin-audit',
  'admin-analytics', 'admin-finance', 'admin-content', 'admin-support',
  'system', 'security', 'audit'
];

const VALID_ACTIONS = [
  'read', 'write', 'create', 'update', 'delete', 'view', 'admin', 'manage',
  'execute', 'export', 'import', 'grant', 'revoke', 'audit', 'all'
];

const VALID_SCOPES = ['own', 'team', 'org', 'system', 'all'];

// Legacy patterns to detect and flag
const LEGACY_PATTERNS = [
  /\w+\.\w+/, // dot notation like "users.view"
  /['"][^'"]*:(\w+)['"]/, // colon without domain:action:scope format
  /actions:\s*\[.*\]/, // legacy actions array
];

class PermissionValidator {
  constructor() {
    this.errors = [];
    this.warnings = [];
    this.permissionsFound = [];
  }

  validateFile(filePath) {
    const content = fs.readFileSync(filePath, 'utf8');
    const relativePath = path.relative(process.cwd(), filePath);
    
    console.log(`🔍 Validating ${relativePath}`);
    
    // Extract potential permissions from file
    const permissions = this.extractPermissions(content);
    
    permissions.forEach(permission => {
      this.validatePermission(permission.value, relativePath, permission.line);
    });
    
    // Check for legacy patterns
    this.checkLegacyPatterns(content, relativePath);
  }
  
  extractPermissions(content) {
    const permissions = [];
    const lines = content.split('\n');
    
    lines.forEach((line, index) => {
      // Look for string patterns that might be permissions
      const matches = [
        ...line.matchAll(/['"]([a-z-]+:[a-z]+:[a-z]+)['"/\s\]\},]/g),
        ...line.matchAll(/['"]([a-z-]+\.[a-z]+)['"/\s\]\},]/g), // legacy format
        ...line.matchAll(/id:\s*['"]([^'"]+)['"].*permission/gi),
      ];
      
      matches.forEach(match => {
        if (match[1]) {
          permissions.push({
            value: match[1],
            line: index + 1,
          });
        }
      });
    });
    
    return permissions;
  }
  
  validatePermission(permission, filePath, lineNumber) {
    this.permissionsFound.push({ permission, filePath, lineNumber });
    
    // Check if it's unified format
    if (!UNIFIED_FORMAT_REGEX.test(permission)) {
      if (permission.includes('.')) {
        this.warnings.push({
          type: 'legacy_format',
          message: `Legacy dot notation found: "${permission}"`,
          file: filePath,
          line: lineNumber,
          suggestion: `Convert to unified format: "${this.suggestUnifiedFormat(permission)}"`
        });
      } else {
        this.errors.push({
          type: 'invalid_format',
          message: `Invalid permission format: "${permission}"`,
          file: filePath,
          line: lineNumber,
          expected: 'domain:action:scope'
        });
      }
      return;
    }
    
    // Validate parts
    const [domain, action, scope] = permission.split(':');
    
    if (!VALID_DOMAINS.includes(domain) && domain !== '*') {
      this.errors.push({
        type: 'invalid_domain',
        message: `Invalid domain: "${domain}" in permission "${permission}"`,
        file: filePath,
        line: lineNumber,
        validDomains: VALID_DOMAINS
      });
    }
    
    if (!VALID_ACTIONS.includes(action) && action !== '*') {
      this.errors.push({
        type: 'invalid_action', 
        message: `Invalid action: "${action}" in permission "${permission}"`,
        file: filePath,
        line: lineNumber,
        validActions: VALID_ACTIONS
      });
    }
    
    if (!VALID_SCOPES.includes(scope) && scope !== '*') {
      this.errors.push({
        type: 'invalid_scope',
        message: `Invalid scope: "${scope}" in permission "${permission}"`,
        file: filePath,
        line: lineNumber,
        validScopes: VALID_SCOPES
      });
    }
    
    // Style recommendations
    this.checkStyle(permission, filePath, lineNumber);
  }
  
  checkStyle(permission, filePath, lineNumber) {
    const [domain, action, scope] = permission.split(':');
    
    // Recommend "read" over "view" for consistency
    if (action === 'view') {
      this.warnings.push({
        type: 'style_recommendation',
        message: `Consider using "read" instead of "view" in "${permission}"`,
        file: filePath,
        line: lineNumber,
        suggestion: permission.replace(':view:', ':read:')
      });
    }
    
    // Recommend admin domains for admin actions
    if (action === 'admin' && !domain.startsWith('admin-') && !['system', 'security'].includes(domain)) {
      this.warnings.push({
        type: 'style_recommendation',
        message: `Consider using admin-prefixed domain for admin action in "${permission}"`,
        file: filePath,
        line: lineNumber
      });
    }
    
    // Warn about overly broad permissions
    if (scope === 'all' && action !== 'admin') {
      this.warnings.push({
        type: 'security_warning',
        message: `Overly broad scope "all" in "${permission}"`,
        file: filePath,
        line: lineNumber,
        suggestion: 'Consider using a more specific scope for better security'
      });
    }
  }
  
  checkLegacyPatterns(content, filePath) {
    const lines = content.split('\n');
    
    lines.forEach((line, index) => {
      LEGACY_PATTERNS.forEach(pattern => {
        if (pattern.test(line)) {
          this.warnings.push({
            type: 'legacy_pattern',
            message: 'Legacy permission pattern detected',
            file: filePath,
            line: index + 1,
            code: line.trim()
          });
        }
      });
    });
  }
  
  suggestUnifiedFormat(legacyPermission) {
    if (legacyPermission.includes('.')) {
      const [domain, action] = legacyPermission.split('.');
      const unifiedAction = action === 'view' ? 'read' : action;
      const scope = action === 'admin' ? 'system' : 'own';
      return `${domain}:${unifiedAction}:${scope}`;
    }
    return 'domain:action:scope';
  }
  
  printReport() {
    console.log('\n📊 Permission Validation Report');
    console.log('==================================');
    
    console.log(`\n✅ Permissions found: ${this.permissionsFound.length}`);
    console.log(`❌ Errors: ${this.errors.length}`);
    console.log(`⚠️  Warnings: ${this.warnings.length}`);
    
    if (this.errors.length > 0) {
      console.log('\n🚨 ERRORS:');
      this.errors.forEach(error => {
        console.log(`  ❌ ${error.file}:${error.line} - ${error.message}`);
        if (error.expected) console.log(`     Expected format: ${error.expected}`);
        if (error.validDomains) console.log(`     Valid domains: ${error.validDomains.slice(0, 5).join(', ')}...`);
      });
    }
    
    if (this.warnings.length > 0) {
      console.log('\n⚠️  WARNINGS:');
      this.warnings.forEach(warning => {
        console.log(`  ⚠️  ${warning.file}:${warning.line} - ${warning.message}`);
        if (warning.suggestion) console.log(`     Suggestion: ${warning.suggestion}`);
        if (warning.code) console.log(`     Code: ${warning.code}`);
      });
    }
    
    // Style recommendations summary
    const styleIssues = this.warnings.filter(w => w.type === 'style_recommendation').length;
    const securityWarnings = this.warnings.filter(w => w.type === 'security_warning').length;
    const legacyPatterns = this.warnings.filter(w => w.type === 'legacy_pattern').length;
    
    console.log('\n📈 Issue Summary:');
    console.log(`  • Style recommendations: ${styleIssues}`);
    console.log(`  • Security warnings: ${securityWarnings}`);
    console.log(`  • Legacy patterns: ${legacyPatterns}`);
    
    // Success criteria
    const hasErrors = this.errors.length > 0;
    const hasLegacyPatterns = legacyPatterns > 0;
    
    console.log('\n🎯 Status:');
    if (!hasErrors && !hasLegacyPatterns) {
      console.log('  ✅ All permissions use unified format!');
    } else {
      console.log('  ❌ Issues found. Run cleanup to fix legacy patterns.');
    }
    
    return !hasErrors;
  }
}

// Main execution
function main() {
  const validator = new PermissionValidator();
  
  console.log('🚀 Starting Permission Format Validation');
  console.log('=========================================');
  
  // Find all relevant files
  const patterns = [
    'apps/*/types/**/*.ts',
    'apps/*/config/**/*.ts',
    'apps/*/src/**/*.ts',
    'apps/*/src/**/*.rs',
  ];
  
  const files = [];
  patterns.forEach(pattern => {
    files.push(...glob.sync(pattern, { ignore: 'node_modules/**' }));
  });
  
  console.log(`📁 Found ${files.length} files to check`);
  
  // Validate each file
  files.forEach(file => {
    try {
      validator.validateFile(file);
    } catch (error) {
      console.error(`Error processing ${file}:`, error.message);
    }
  });
  
  // Print results
  const success = validator.printReport();
  
  // Exit with appropriate code
  process.exit(success ? 0 : 1);
}

if (require.main === module) {
  main();
}

module.exports = { PermissionValidator };