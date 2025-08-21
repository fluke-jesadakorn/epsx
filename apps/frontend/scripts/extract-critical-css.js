/**
 * Critical CSS Extraction Script
 * 
 * This script extracts critical CSS for above-the-fold content to improve
 * First Contentful Paint (FCP) and Largest Contentful Paint (LCP) metrics.
 * 
 * It analyzes the application's routes and extracts the minimum CSS needed
 * for initial render, which can then be inlined in the HTML head.
 * 
 * Benefits:
 * - Faster initial page load
 * - Better Core Web Vitals scores
 * - Reduced render-blocking resources
 * - Improved user experience
 */

const fs = require('fs').promises;
const path = require('path');
const critical = require('critical');

// Configuration for critical CSS extraction
const criticalConfig = {
  // Base configuration
  base: path.join(__dirname, '../'),
  src: 'index.html',
  target: {
    css: 'styles/critical-extracted.css',
    html: 'index-critical.html',
    uncritical: 'styles/non-critical.css'
  },
  
  // Viewport dimensions for critical CSS calculation
  dimensions: [
    {
      height: 900,
      width: 1200
    },
    {
      height: 800,
      width: 768
    },
    {
      height: 812,
      width: 375
    }
  ],
  
  // Performance settings
  penthouse: {
    blockJSRequests: false,
    forceInclude: [
      // Always include these selectors in critical CSS
      '.loading-critical',
      '.layout-stable',
      '.container-responsive',
      '.nav-critical',
      '.hero-critical',
      '.btn-critical',
      '.card-critical',
      '.text-critical-heading',
      '.text-critical-subheading',
      '.text-gradient-critical',
      
      // Font loading classes
      '.font-loading',
      '.font-loaded',
      
      // Theme classes
      '.light',
      '.dark',
      
      // Critical animations
      '@keyframes loading-shimmer',
      '@keyframes pulse',
      '@keyframes spin',
    ],
    
    // Properties to include
    include: [
      /^\.loading/,
      /^\.critical/,
      /^\.hero/,
      /^\.nav/,
      /^\.container/,
      /^\.btn-critical/,
      /^\.card-critical/,
      /^\.text-critical/,
      /^\.skeleton/,
      /^\.spinner/,
    ],
    
    // Timeout for page analysis
    timeout: 30000,
    maxEmbeddedBase64Length: 1000,
    userAgent: 'Mozilla/5.0 (compatible; Critical CSS Generator)',
  },
  
  // CSS optimization
  minify: true,
  extract: true,
  inlineImages: false,
  
  // Ignore certain CSS rules
  ignore: {
    atrule: ['@font-face', '@import'],
    rule: [
      /^\.non-critical/,
      /^\.development-only/,
      /^\.debug/,
      /hover/,
      /focus/,
      /active/,
      /print/,
    ],
    decl: (node, value) => {
      // Ignore large background images
      return /^background-image/.test(node.prop) && value.length > 1000;
    }
  }
};

// Routes to analyze for critical CSS
const routes = [
  {
    name: 'home',
    url: 'http://localhost:3000/',
    output: 'critical-home.css'
  },
  {
    name: 'login',
    url: 'http://localhost:3000/login',
    output: 'critical-login.css'
  },
  {
    name: 'dashboard',
    url: 'http://localhost:3000/dashboard',
    output: 'critical-dashboard.css'
  },
  {
    name: 'analytics',
    url: 'http://localhost:3000/analytics',
    output: 'critical-analytics.css'
  },
  {
    name: 'trading',
    url: 'http://localhost:3000/trading',
    output: 'critical-trading.css'
  }
];

/**
 * Extract critical CSS for a specific route
 */
async function extractCriticalForRoute(route) {
  console.log(`📊 Analyzing critical CSS for ${route.name}...`);
  
  try {
    const result = await critical.generate({
      ...criticalConfig,
      src: route.url,
      target: {
        css: path.join(__dirname, '..', 'styles', 'critical', route.output)
      },
      width: 1200,
      height: 900,
    });
    
    // Save the critical CSS
    const outputPath = path.join(__dirname, '..', 'styles', 'critical', route.output);
    await fs.writeFile(outputPath, result.css);
    
    // Calculate size reduction
    const originalSize = result.originalCss?.length || 0;
    const criticalSize = result.css.length;
    const reduction = originalSize > 0 ? ((originalSize - criticalSize) / originalSize * 100).toFixed(1) : 0;
    
    console.log(`✅ ${route.name}: ${criticalSize} bytes critical CSS (${reduction}% reduction)`);
    
    return {
      route: route.name,
      criticalSize,
      originalSize,
      reduction: `${reduction}%`,
      output: route.output
    };
    
  } catch (error) {
    console.error(`❌ Error extracting critical CSS for ${route.name}:`, error.message);
    return {
      route: route.name,
      error: error.message
    };
  }
}

/**
 * Generate a combined critical CSS file for common elements
 */
async function generateCommonCriticalCSS() {
  console.log('🔄 Generating common critical CSS...');
  
  // Read the critical CSS file we created
  const criticalCSSPath = path.join(__dirname, '..', 'styles', 'critical.css');
  
  try {
    const criticalCSS = await fs.readFile(criticalCSSPath, 'utf8');
    
    // Extract only the most critical parts
    const commonCritical = criticalCSS
      .split('\n')
      .filter(line => {
        // Include essential utilities and base styles
        return line.includes('.container-responsive') ||
               line.includes('.loading-critical') ||
               line.includes('.layout-stable') ||
               line.includes('.nav-critical') ||
               line.includes('.btn-critical') ||
               line.includes('.card-critical') ||
               line.includes('.text-critical') ||
               line.includes('.skeleton') ||
               line.includes('.spinner') ||
               line.includes('@keyframes') ||
               line.includes(':root') ||
               line.includes('body') ||
               line.includes('html');
      })
      .join('\n');
    
    const outputPath = path.join(__dirname, '..', 'styles', 'critical', 'common.css');
    await fs.writeFile(outputPath, commonCritical);
    
    console.log(`✅ Common critical CSS generated: ${commonCritical.length} bytes`);
    
    return {
      size: commonCritical.length,
      output: 'common.css'
    };
    
  } catch (error) {
    console.error('❌ Error generating common critical CSS:', error.message);
    return { error: error.message };
  }
}

/**
 * Create critical CSS directory structure
 */
async function createCriticalDirectories() {
  const dirs = [
    path.join(__dirname, '..', 'styles', 'critical'),
    path.join(__dirname, '..', 'styles', 'non-critical'),
  ];
  
  for (const dir of dirs) {
    try {
      await fs.mkdir(dir, { recursive: true });
    } catch (error) {
      // Directory might already exist
    }
  }
}

/**
 * Generate critical CSS report
 */
async function generateReport(results, commonResult) {
  const report = {
    timestamp: new Date().toISOString(),
    summary: {
      totalRoutes: results.length,
      successfulExtractions: results.filter(r => !r.error).length,
      failedExtractions: results.filter(r => r.error).length,
      totalCriticalSize: results.reduce((sum, r) => sum + (r.criticalSize || 0), 0),
      averageReduction: results
        .filter(r => !r.error && r.reduction)
        .reduce((sum, r) => sum + parseFloat(r.reduction), 0) / results.filter(r => !r.error).length || 0
    },
    routes: results,
    common: commonResult,
    recommendations: []
  };
  
  // Add recommendations based on results
  if (report.summary.averageReduction < 50) {
    report.recommendations.push('Consider optimizing CSS structure for better critical CSS extraction');
  }
  
  if (report.summary.totalCriticalSize > 50000) {
    report.recommendations.push('Critical CSS size is large, consider further optimization');
  }
  
  if (report.summary.failedExtractions > 0) {
    report.recommendations.push('Some routes failed extraction, check server availability and route accessibility');
  }
  
  // Save report
  const reportPath = path.join(__dirname, '..', 'styles', 'critical', 'extraction-report.json');
  await fs.writeFile(reportPath, JSON.stringify(report, null, 2));
  
  return report;
}

/**
 * Create Next.js integration helper
 */
async function createNextJSIntegration() {
  const integrationCode = `
/**
 * Next.js Critical CSS Integration
 * 
 * This file provides utilities to integrate critical CSS extraction
 * with Next.js for optimal performance.
 */

import { readFileSync } from 'fs';
import { join } from 'path';

const criticalCSSCache = new Map();

/**
 * Get critical CSS for a specific route
 */
export function getCriticalCSS(route: string): string {
  if (criticalCSSCache.has(route)) {
    return criticalCSSCache.get(route);
  }
  
  try {
    const filePath = join(process.cwd(), 'styles', 'critical', \`critical-\${route}.css\`);
    const css = readFileSync(filePath, 'utf8');
    criticalCSSCache.set(route, css);
    return css;
  } catch (error) {
    console.warn(\`Could not load critical CSS for route: \${route}\`);
    return '';
  }
}

/**
 * Get common critical CSS
 */
export function getCommonCriticalCSS(): string {
  if (criticalCSSCache.has('common')) {
    return criticalCSSCache.get('common');
  }
  
  try {
    const filePath = join(process.cwd(), 'styles', 'critical', 'common.css');
    const css = readFileSync(filePath, 'utf8');
    criticalCSSCache.set('common', css);
    return css;
  } catch (error) {
    console.warn('Could not load common critical CSS');
    return '';
  }
}

/**
 * Critical CSS component for Next.js
 */
export function CriticalCSS({ route }: { route?: string }) {
  const commonCSS = getCommonCriticalCSS();
  const routeCSS = route ? getCriticalCSS(route) : '';
  
  const combinedCSS = [commonCSS, routeCSS].filter(Boolean).join('\\n');
  
  if (!combinedCSS) return null;
  
  return (
    <style
      dangerouslySetInnerHTML={{
        __html: combinedCSS
      }}
      data-critical-css
    />
  );
}
`;

  const outputPath = path.join(__dirname, '..', 'lib', 'critical-css.tsx');
  await fs.writeFile(outputPath, integrationCode);
  
  console.log('✅ Next.js integration helper created');
}

/**
 * Main execution function
 */
async function main() {
  console.log('🚀 Starting critical CSS extraction...\n');
  
  try {
    // Create necessary directories
    await createCriticalDirectories();
    
    // Generate common critical CSS first
    const commonResult = await generateCommonCriticalCSS();
    
    // Extract critical CSS for each route
    const results = [];
    for (const route of routes) {
      const result = await extractCriticalForRoute(route);
      results.push(result);
    }
    
    // Generate report
    const report = await generateReport(results, commonResult);
    
    // Create Next.js integration
    await createNextJSIntegration();
    
    // Print summary
    console.log('\n📋 Critical CSS Extraction Complete!');
    console.log('==========================================');
    console.log(\`Total Routes Analyzed: \${report.summary.totalRoutes}\`);
    console.log(\`Successful Extractions: \${report.summary.successfulExtractions}\`);
    console.log(\`Average Size Reduction: \${report.summary.averageReduction.toFixed(1)}%\`);
    console.log(\`Total Critical CSS Size: \${report.summary.totalCriticalSize} bytes\`);
    
    if (report.recommendations.length > 0) {
      console.log('\\n💡 Recommendations:');
      report.recommendations.forEach(rec => console.log(\`  • \${rec}\`));
    }
    
    console.log('\\n📁 Output Files:');
    console.log('  • styles/critical/common.css - Common critical CSS');
    console.log('  • styles/critical/critical-*.css - Route-specific critical CSS');
    console.log('  • styles/critical/extraction-report.json - Detailed report');
    console.log('  • lib/critical-css.tsx - Next.js integration helper');
    
  } catch (error) {
    console.error('❌ Critical CSS extraction failed:', error.message);
    process.exit(1);
  }
}

// Run if this file is executed directly
if (require.main === module) {
  main().catch(console.error);
}

module.exports = {
  extractCriticalForRoute,
  generateCommonCriticalCSS,
  createNextJSIntegration,
  routes,
  criticalConfig
};
`;

const packageJsonUpdate = {
  "scripts": {
    "extract-critical": "node scripts/extract-critical-css.js",
    "build:critical": "npm run extract-critical && npm run build",
    "analyze:css": "npm run extract-critical && npx bundlesize"
  },
  "devDependencies": {
    "critical": "^6.0.0",
    "bundlesize": "^0.18.1"
  }
};

console.log('📦 Add these to your package.json:');
console.log(JSON.stringify(packageJsonUpdate, null, 2));