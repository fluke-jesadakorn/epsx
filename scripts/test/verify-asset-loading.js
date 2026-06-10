#!/usr/bin/env node

/**
 * EPSX Asset Loading Verification Script
 * Tests CSS, JavaScript, and static asset loading for deployed services
 */

const https = require('https');
const http = require('http');

// Service configurations
const SERVICES = {
  'Frontend (Dev)': 'https://epsx-frontend-dev-307278481624.us-central1.run.app',
  'Admin (Dev)': 'https://epsx-admin-dev-307278481624.us-central1.run.app'
};

// Asset types to test
const ASSET_TESTS = [
  // CSS files (these should load after fix)
  { path: '/_next/static/css/e7d449e845b9c0e5.css', type: 'CSS', expected: 200 },
  { path: '/_next/static/css/67105386d17d24c3.css', type: 'CSS', expected: 200 },
  
  // JavaScript files
  { path: '/_next/static/chunks/webpack-c757251fe6896ba5.js', type: 'JS', expected: 200 },
  { path: '/_next/static/chunks/main-app-b760bd37a099fdd1.js', type: 'JS', expected: 200 },
  
  // Public assets
  { path: '/favicon.ico', type: 'Icon', expected: 200 },
  
  // Font files (common patterns)
  { path: '/_next/static/media/07c22f4624039e44-s.p.woff2', type: 'Font', expected: 200 },
];

function makeRequest(url, method = 'HEAD') {
  return new Promise((resolve) => {
    const client = url.startsWith('https') ? https : http;
    const startTime = Date.now();
    
    const req = client.request(url, { method, timeout: 10000 }, (res) => {
      let data = '';
      if (method === 'GET') {
        res.on('data', chunk => data += chunk);
      }
      
      res.on('end', () => {
        resolve({
          url,
          status: res.statusCode,
          headers: res.headers,
          responseTime: Date.now() - startTime,
          contentLength: res.headers['content-length'] || data.length,
          contentType: res.headers['content-type'],
          cacheControl: res.headers['cache-control'],
          body: method === 'GET' ? data : null
        });
      });
    });
    
    req.on('error', (error) => {
      resolve({
        url,
        status: 'ERROR',
        error: error.message,
        responseTime: Date.now() - startTime
      });
    });
    
    req.on('timeout', () => {
      req.destroy();
      resolve({
        url,
        status: 'TIMEOUT',
        responseTime: Date.now() - startTime
      });
    });
    
    req.end();
  });
}

function getStatusIcon(status, expected = 200) {
  if (status === expected) return '✅';
  if (status === 404) return '❌';
  if (status === 'ERROR') return '🔥';
  if (status === 'TIMEOUT') return '⏰';
  if (status >= 300 && status < 400) return '↗️';
  if (status >= 400) return '⚠️';
  return '❓';
}

function formatSize(bytes) {
  if (!bytes || bytes === 0) return 'Unknown';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

async function testService(serviceName, baseUrl) {
  console.log(`\n🔍 Testing ${serviceName}`);
  console.log(`📡 Base URL: ${baseUrl}`);
  console.log('-'.repeat(100));
  
  const results = {
    total: 0,
    passed: 0,
    failed: 0,
    errors: 0
  };
  
  for (const asset of ASSET_TESTS) {
    const fullUrl = baseUrl + asset.path;
    const result = await makeRequest(fullUrl, 'HEAD');
    
    results.total++;
    const icon = getStatusIcon(result.status, asset.expected);
    
    if (result.status === asset.expected) {
      results.passed++;
    } else if (result.status === 'ERROR' || result.status === 'TIMEOUT') {
      results.errors++;
    } else {
      results.failed++;
    }
    
    console.log(`${icon} ${asset.type.padEnd(6)} ${result.status.toString().padEnd(7)} ${result.responseTime}ms  ${asset.path}`);
    
    if (result.contentType) {
      console.log(`       Content-Type: ${result.contentType}`);
    }
    
    if (result.contentLength && result.contentLength !== '0') {
      console.log(`       Size: ${formatSize(parseInt(result.contentLength))}`);
    }
    
    if (result.cacheControl) {
      console.log(`       Cache: ${result.cacheControl}`);
    }
    
    if (result.error) {
      console.log(`       Error: ${result.error}`);
    }
    
    console.log();
  }
  
  // Summary
  console.log('📊 Summary:');
  console.log(`   ✅ Passed: ${results.passed}/${results.total}`);
  console.log(`   ❌ Failed: ${results.failed}/${results.total}`);
  console.log(`   🔥 Errors: ${results.errors}/${results.total}`);
  
  const successRate = ((results.passed / results.total) * 100).toFixed(1);
  console.log(`   📈 Success Rate: ${successRate}%`);
  
  return results;
}

async function runVerification() {
  console.log('🚀 EPSX Asset Loading Verification');
  console.log('==================================');
  console.log(`⏰ Started at: ${new Date().toISOString()}`);
  
  const allResults = [];
  
  for (const [serviceName, baseUrl] of Object.entries(SERVICES)) {
    const results = await testService(serviceName, baseUrl);
    allResults.push({ serviceName, results });
  }
  
  console.log('\n🏁 Overall Results');
  console.log('===================');
  
  let totalPassed = 0;
  let totalTests = 0;
  
  for (const { serviceName, results } of allResults) {
    totalPassed += results.passed;
    totalTests += results.total;
    
    const rate = ((results.passed / results.total) * 100).toFixed(1);
    const status = results.passed === results.total ? '✅' : '⚠️';
    
    console.log(`${status} ${serviceName}: ${results.passed}/${results.total} (${rate}%)`);
  }
  
  const overallRate = ((totalPassed / totalTests) * 100).toFixed(1);
  console.log(`\n📊 Overall Success Rate: ${totalPassed}/${totalTests} (${overallRate}%)`);
  
  if (overallRate === '100.0') {
    console.log('\n🎉 All asset loading tests passed! Services are working correctly.');
  } else if (overallRate >= '80.0') {
    console.log('\n⚠️  Most assets are loading, but some issues detected. Check failed tests above.');
  } else {
    console.log('\n🚨 Multiple asset loading issues detected. Review Dockerfile static file configuration.');
  }
  
  console.log(`\n⏱️  Completed at: ${new Date().toISOString()}`);
  
  // Exit with appropriate code
  process.exit(overallRate === '100.0' ? 0 : 1);
}

// Run the verification
runVerification().catch((error) => {
  console.error('🔥 Verification failed:', error);
  process.exit(1);
});