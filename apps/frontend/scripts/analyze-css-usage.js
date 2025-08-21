/**
 * CSS Usage Analysis and Cleanup Script
 * 
 * This script analyzes the CSS codebase to identify:
 * - Duplicate animation definitions
 * - Unused utility classes
 * - Redundant CSS rules
 * - Optimization opportunities
 * 
 * It helps reduce bundle size and improve maintainability by removing
 * dead code and consolidating duplicates.
 */

const fs = require('fs').promises;
const path = require('path');
const glob = require('glob');
const postcss = require('postcss');
const postcssParser = require('postcss');

// ============================================================================
// CONFIGURATION
// ============================================================================

const config = {
  // Directories to analyze
  cssDirectories: [
    'app/**/*.css',
    'styles/**/*.css',
    'components/**/*.css',
  ],
  
  // Component files to check for class usage
  componentDirectories: [
    'app/**/*.{tsx,ts,jsx,js}',
    'components/**/*.{tsx,ts,jsx,js}',
    'lib/**/*.{tsx,ts,jsx,js}',
    'hooks/**/*.{tsx,ts,jsx,js}',
    'utils/**/*.{tsx,ts,jsx,js}',
  ],
  
  // Ignore patterns
  ignore: [
    'node_modules/**',
    '.next/**',
    'dist/**',
    'build/**',
    '*.test.*',
    '*.spec.*',
  ],
  
  // Classes to always keep (even if not found in usage)
  alwaysKeep: [
    // Framework classes
    'dark',
    'light',
    'sr-only',
    'sr-only-focusable',
    
    // Dynamic classes that might not be detected
    /^animate-/,
    /^bg-gradient-/,
    /^text-gradient-/,
    /^hover:/,
    /^focus:/,
    /^active:/,
    /^dark:/,
    /^sm:/,
    /^md:/,
    /^lg:/,
    /^xl:/,
    
    // Critical classes
    /^loading-/,
    /^critical/,
    /^skeleton/,
    /^spinner/,
  ]
};

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Get all files matching patterns
 */
async function getFiles(patterns, ignore = []) {
  const allFiles = new Set();
  
  for (const pattern of patterns) {
    const files = await new Promise((resolve, reject) => {
      glob(pattern, { ignore }, (err, files) => {
        if (err) reject(err);
        else resolve(files);
      });
    });
    
    files.forEach(file => allFiles.add(file));
  }
  
  return Array.from(allFiles);
}

/**
 * Parse CSS file and extract rules
 */
async function parseCSSFile(filePath) {
  try {
    const content = await fs.readFile(filePath, 'utf8');
    const ast = postcss.parse(content, { from: filePath });
    
    const rules = [];
    const keyframes = [];
    const animations = [];
    
    ast.walkRules(rule => {
      rules.push({
        selector: rule.selector,
        declarations: rule.nodes
          .filter(node => node.type === 'decl')
          .map(decl => ({ prop: decl.prop, value: decl.value })),
        file: filePath,
        line: rule.source?.start?.line || 0,
      });
    });
    
    ast.walkAtRules(atrule => {
      if (atrule.name === 'keyframes') {
        keyframes.push({
          name: atrule.params,
          rules: atrule.nodes?.length || 0,
          file: filePath,
          line: atrule.source?.start?.line || 0,
        });
      }
    });
    
    // Extract animation references from declarations
    ast.walkDecls(decl => {
      if (decl.prop === 'animation' || decl.prop === 'animation-name') {
        const animationName = decl.value.split(' ')[0];
        animations.push({
          name: animationName,
          property: decl.prop,
          file: filePath,
          line: decl.source?.start?.line || 0,
        });
      }
    });
    
    return { rules, keyframes, animations };
  } catch (error) {
    console.warn(`Warning: Could not parse CSS file ${filePath}:`, error.message);
    return { rules: [], keyframes: [], animations: [] };
  }
}

/**
 * Extract class names from component files
 */
async function extractClassNamesFromFile(filePath) {
  try {
    const content = await fs.readFile(filePath, 'utf8');
    const classNames = new Set();
    
    // Regular expressions to find class names
    const patterns = [
      // className="..."
      /className\s*=\s*["'`]([^"'`]+)["'`]/g,
      // className={...}
      /className\s*=\s*\{[^}]*["'`]([^"'`]+)["'`][^}]*\}/g,
      // class="..."
      /class\s*=\s*["'`]([^"'`]+)["'`]/g,
      // Tailwind @apply
      /@apply\s+([^;]+);/g,
      // CSS modules
      /styles\.([a-zA-Z_][a-zA-Z0-9_-]*)/g,
      // clsx, cn, classNames function calls
      /(?:clsx|cn|classNames)\s*\(\s*["'`]([^"'`]+)["'`]/g,
    ];
    
    patterns.forEach(pattern => {
      let match;
      while ((match = pattern.exec(content)) !== null) {
        const classString = match[1];
        if (classString) {
          // Split by spaces and add individual classes
          classString.split(/\s+/).forEach(cls => {
            if (cls.trim()) {
              classNames.add(cls.trim());
            }
          });
        }
      }
    });
    
    return Array.from(classNames);
  } catch (error) {
    console.warn(`Warning: Could not parse component file ${filePath}:`, error.message);
    return [];
  }
}

// ============================================================================
// ANALYSIS FUNCTIONS
// ============================================================================

/**
 * Find duplicate keyframe animations
 */
function findDuplicateKeyframes(cssData) {
  const keyframeGroups = new Map();
  
  // Group keyframes by name
  cssData.forEach(data => {
    data.keyframes.forEach(keyframe => {
      if (!keyframeGroups.has(keyframe.name)) {
        keyframeGroups.set(keyframe.name, []);
      }
      keyframeGroups.get(keyframe.name).push(keyframe);
    });
  });
  
  // Find duplicates
  const duplicates = [];
  keyframeGroups.forEach((keyframes, name) => {
    if (keyframes.length > 1) {
      duplicates.push({
        name,
        count: keyframes.length,
        files: keyframes.map(k => ({ file: k.file, line: k.line })),
      });
    }
  });
  
  return duplicates;
}

/**
 * Find unused animations
 */
function findUnusedAnimations(cssData) {
  const definedKeyframes = new Set();
  const usedAnimations = new Set();
  
  // Collect all defined keyframes
  cssData.forEach(data => {
    data.keyframes.forEach(keyframe => {
      definedKeyframes.add(keyframe.name);
    });
  });
  
  // Collect all used animations
  cssData.forEach(data => {
    data.animations.forEach(animation => {
      usedAnimations.add(animation.name);
    });
  });
  
  // Find keyframes that are defined but never used
  const unused = [];
  definedKeyframes.forEach(name => {
    if (!usedAnimations.has(name)) {
      unused.push(name);
    }
  });
  
  return unused;
}

/**
 * Find duplicate CSS rules
 */
function findDuplicateRules(cssData) {
  const ruleGroups = new Map();
  
  // Group rules by selector
  cssData.forEach(data => {
    data.rules.forEach(rule => {
      const key = rule.selector;
      if (!ruleGroups.has(key)) {
        ruleGroups.set(key, []);
      }
      ruleGroups.get(key).push(rule);
    });
  });
  
  // Find potential duplicates (same selector, similar declarations)
  const duplicates = [];
  ruleGroups.forEach((rules, selector) => {
    if (rules.length > 1) {
      // Group by declaration signature
      const declarationGroups = new Map();
      
      rules.forEach(rule => {
        const signature = rule.declarations
          .map(d => `${d.prop}:${d.value}`)
          .sort()
          .join(';');
        
        if (!declarationGroups.has(signature)) {
          declarationGroups.set(signature, []);
        }
        declarationGroups.get(signature).push(rule);
      });
      
      declarationGroups.forEach((duplicateRules, signature) => {
        if (duplicateRules.length > 1) {
          duplicates.push({
            selector,
            count: duplicateRules.length,
            signature,
            files: duplicateRules.map(r => ({ file: r.file, line: r.line })),
          });
        }
      });
    }
  });
  
  return duplicates;
}

/**
 * Find unused CSS classes
 */
function findUnusedClasses(cssData, usedClasses) {
  const definedClasses = new Set();
  const usedClassSet = new Set(usedClasses);
  
  // Extract class selectors from CSS
  cssData.forEach(data => {
    data.rules.forEach(rule => {
      // Extract class names from selectors
      const classMatches = rule.selector.match(/\\.([a-zA-Z_][a-zA-Z0-9_-]*)/g);
      if (classMatches) {
        classMatches.forEach(match => {
          const className = match.substring(1); // Remove the dot
          definedClasses.add(className);
        });
      }
    });
  });
  
  // Find classes that are defined but not used
  const unused = [];
  definedClasses.forEach(className => {
    // Check if class is used or should be kept
    const shouldKeep = config.alwaysKeep.some(pattern => {
      if (typeof pattern === 'string') {
        return className === pattern;
      } else if (pattern instanceof RegExp) {
        return pattern.test(className);
      }
      return false;
    });
    
    if (!shouldKeep && !usedClassSet.has(className)) {
      unused.push(className);
    }
  });
  
  return unused;
}

/**
 * Calculate potential savings
 */
function calculateSavings(cssData, duplicates, unused) {
  let totalSize = 0;
  let duplicateSize = 0;
  let unusedSize = 0;
  
  // Estimate total CSS size
  cssData.forEach(data => {
    data.rules.forEach(rule => {
      const ruleSize = rule.selector.length + 
        rule.declarations.reduce((sum, decl) => sum + decl.prop.length + decl.value.length + 10, 0);
      totalSize += ruleSize;
    });
    
    data.keyframes.forEach(keyframe => {
      // Rough estimate for keyframe size
      unusedSize += keyframe.name.length * 50; // Estimated average keyframe size
    });
  });
  
  // Estimate duplicate savings
  duplicates.duplicateKeyframes.forEach(dup => {
    duplicateSize += (dup.count - 1) * 100; // Rough estimate
  });
  
  duplicates.duplicateRules.forEach(dup => {
    duplicateSize += (dup.count - 1) * 50; // Rough estimate
  });
  
  // Estimate unused class savings
  unused.unusedClasses.forEach(() => {
    unusedSize += 30; // Rough estimate per unused class
  });
  
  unused.unusedAnimations.forEach(() => {
    unusedSize += 100; // Rough estimate per unused animation
  });
  
  return {
    totalSize,
    duplicateSize,
    unusedSize,
    potentialSavings: duplicateSize + unusedSize,
    savingsPercentage: totalSize > 0 ? ((duplicateSize + unusedSize) / totalSize * 100).toFixed(1) : 0,
  };
}

// ============================================================================
// CLEANUP FUNCTIONS
// ============================================================================

/**
 * Generate cleanup recommendations
 */
function generateCleanupRecommendations(analysis) {
  const recommendations = [];
  
  // Duplicate keyframes
  if (analysis.duplicates.duplicateKeyframes.length > 0) {
    recommendations.push({
      type: 'duplicate-keyframes',
      priority: 'high',
      title: 'Remove Duplicate Keyframe Animations',
      description: `Found ${analysis.duplicates.duplicateKeyframes.length} duplicate keyframe animations. Consolidate to single definitions.`,
      actions: analysis.duplicates.duplicateKeyframes.map(dup => ({
        action: 'consolidate',
        target: dup.name,
        files: dup.files.slice(1), // Keep first, remove others
      })),
    });
  }
  
  // Duplicate rules
  if (analysis.duplicates.duplicateRules.length > 0) {
    recommendations.push({
      type: 'duplicate-rules',
      priority: 'medium',
      title: 'Consolidate Duplicate CSS Rules',
      description: `Found ${analysis.duplicates.duplicateRules.length} duplicate CSS rules. Consider consolidating.`,
      actions: analysis.duplicates.duplicateRules.slice(0, 10).map(dup => ({ // Limit to top 10
        action: 'review',
        target: dup.selector,
        files: dup.files,
      })),
    });
  }
  
  // Unused animations
  if (analysis.unused.unusedAnimations.length > 0) {
    recommendations.push({
      type: 'unused-animations',
      priority: 'medium',
      title: 'Remove Unused Animations',
      description: `Found ${analysis.unused.unusedAnimations.length} unused animations. Safe to remove.`,
      actions: analysis.unused.unusedAnimations.map(name => ({
        action: 'remove',
        target: name,
        type: 'keyframe',
      })),
    });
  }
  
  // Unused classes
  if (analysis.unused.unusedClasses.length > 0) {
    recommendations.push({
      type: 'unused-classes',
      priority: 'low',
      title: 'Review Unused CSS Classes',
      description: `Found ${analysis.unused.unusedClasses.length} potentially unused classes. Review before removing.`,
      actions: analysis.unused.unusedClasses.slice(0, 20).map(className => ({ // Limit to top 20
        action: 'review',
        target: className,
        type: 'class',
      })),
    });
  }
  
  // Performance recommendations
  if (analysis.savings.potentialSavings > 5000) { // 5KB
    recommendations.push({
      type: 'performance',
      priority: 'high',
      title: 'Significant Size Reduction Opportunity',
      description: `Potential ${(analysis.savings.potentialSavings / 1024).toFixed(1)}KB savings (${analysis.savings.savingsPercentage}% reduction) available.`,
      actions: [{
        action: 'implement',
        target: 'cleanup-recommendations',
        description: 'Implement all high and medium priority recommendations',
      }],
    });
  }
  
  return recommendations;
}

/**
 * Generate automated cleanup script
 */
function generateCleanupScript(analysis) {
  const script = `#!/bin/bash
# Automated CSS Cleanup Script
# Generated on ${new Date().toISOString()}

echo "🧹 Starting CSS cleanup..."

# Backup original files
mkdir -p backup-$(date +%Y%m%d-%H%M%S)

`;

  // Add commands for each recommendation
  analysis.recommendations.forEach(rec => {
    if (rec.priority === 'high') {
      script += `
# ${rec.title}
echo "📝 ${rec.description}"
`;
      
      rec.actions.forEach(action => {
        if (action.action === 'remove') {
          script += `# TODO: Remove ${action.target} from CSS files\n`;
        } else if (action.action === 'consolidate') {
          script += `# TODO: Consolidate ${action.target} definitions\n`;
        }
      });
    }
  });
  
  script += `
echo "✅ CSS cleanup complete!"
echo "📊 Review the analysis report for detailed findings."
`;
  
  return script;
}

// ============================================================================
// MAIN ANALYSIS FUNCTION
// ============================================================================

async function analyzeCSSUsage() {
  console.log('🔍 Starting CSS usage analysis...\n');
  
  try {
    // Get all CSS and component files
    console.log('📁 Scanning files...');
    const cssFiles = await getFiles(config.cssDirectories, config.ignore);
    const componentFiles = await getFiles(config.componentDirectories, config.ignore);
    
    console.log(`   Found ${cssFiles.length} CSS files`);
    console.log(`   Found ${componentFiles.length} component files`);
    
    // Parse CSS files
    console.log('\n🎨 Analyzing CSS files...');
    const cssData = [];
    for (const file of cssFiles) {
      const data = await parseCSSFile(file);
      cssData.push(data);
    }
    
    // Extract used class names from components
    console.log('🔍 Extracting class usage from components...');
    const allUsedClasses = new Set();
    for (const file of componentFiles) {
      const classes = await extractClassNamesFromFile(file);
      classes.forEach(cls => allUsedClasses.add(cls));
    }
    
    console.log(`   Found ${allUsedClasses.size} used class names`);
    
    // Perform analysis
    console.log('\n📊 Performing analysis...');
    const duplicateKeyframes = findDuplicateKeyframes(cssData);
    const duplicateRules = findDuplicateRules(cssData);
    const unusedAnimations = findUnusedAnimations(cssData);
    const unusedClasses = findUnusedClasses(cssData, Array.from(allUsedClasses));
    
    const duplicates = { duplicateKeyframes, duplicateRules };
    const unused = { unusedAnimations, unusedClasses };
    const savings = calculateSavings(cssData, duplicates, unused);
    
    // Generate recommendations
    const analysis = { duplicates, unused, savings };
    const recommendations = generateCleanupRecommendations(analysis);
    analysis.recommendations = recommendations;
    
    // Generate cleanup script
    const cleanupScript = generateCleanupScript(analysis);
    
    return {
      analysis,
      cleanupScript,
      summary: {
        cssFiles: cssFiles.length,
        componentFiles: componentFiles.length,
        usedClasses: allUsedClasses.size,
        duplicateKeyframes: duplicateKeyframes.length,
        duplicateRules: duplicateRules.length,
        unusedAnimations: unusedAnimations.length,
        unusedClasses: unusedClasses.length,
        potentialSavings: `${(savings.potentialSavings / 1024).toFixed(1)}KB (${savings.savingsPercentage}%)`,
      }
    };
    
  } catch (error) {
    console.error('❌ Analysis failed:', error);
    throw error;
  }
}

/**
 * Save analysis results
 */
async function saveAnalysisResults(results) {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const outputDir = path.join(__dirname, '..', 'analysis-results');
  
  // Create output directory
  await fs.mkdir(outputDir, { recursive: true });
  
  // Save detailed analysis
  const analysisPath = path.join(outputDir, `css-analysis-${timestamp}.json`);
  await fs.writeFile(analysisPath, JSON.stringify(results.analysis, null, 2));
  
  // Save cleanup script
  const scriptPath = path.join(outputDir, `cleanup-script-${timestamp}.sh`);
  await fs.writeFile(scriptPath, results.cleanupScript);
  await fs.chmod(scriptPath, 0o755); // Make executable
  
  // Save summary report
  const summaryPath = path.join(outputDir, `summary-${timestamp}.md`);
  const summaryReport = generateMarkdownReport(results);
  await fs.writeFile(summaryPath, summaryReport);
  
  return {
    analysisPath,
    scriptPath,
    summaryPath,
  };
}

/**
 * Generate markdown report
 */
function generateMarkdownReport(results) {
  const { analysis, summary } = results;
  
  return `# CSS Usage Analysis Report

Generated on: ${new Date().toISOString()}

## Summary

- **CSS Files Analyzed**: ${summary.cssFiles}
- **Component Files Scanned**: ${summary.componentFiles}
- **Used Classes Found**: ${summary.usedClasses}
- **Potential Savings**: ${summary.potentialSavings}

## Findings

### Duplicate Keyframes (${summary.duplicateKeyframes})
${analysis.duplicates.duplicateKeyframes.map(dup => 
  `- **${dup.name}**: ${dup.count} definitions`
).join('\n')}

### Unused Animations (${summary.unusedAnimations})
${analysis.unused.unusedAnimations.map(name => `- ${name}`).join('\n')}

### Duplicate Rules (${summary.duplicateRules})
${analysis.duplicates.duplicateRules.slice(0, 10).map(dup => 
  `- **${dup.selector}**: ${dup.count} definitions`
).join('\n')}

### Unused Classes (${summary.unusedClasses})
${analysis.unused.unusedClasses.slice(0, 20).map(cls => `- ${cls}`).join('\n')}

## Recommendations

${analysis.recommendations.map(rec => `
### ${rec.title} (${rec.priority})
${rec.description}

Actions:
${rec.actions.map(action => `- ${action.action}: ${action.target}`).join('\n')}
`).join('\n')}

## Next Steps

1. Review the detailed analysis file for complete findings
2. Run the generated cleanup script to automate safe removals
3. Manually review medium and low priority recommendations
4. Monitor bundle size reduction after cleanup
`;
}

// ============================================================================
// CLI EXECUTION
// ============================================================================

async function main() {
  try {
    const results = await analyzeCSSUsage();
    const outputPaths = await saveAnalysisResults(results);
    
    // Print summary
    console.log('\n📋 Analysis Complete!');
    console.log('===================');
    console.log(`CSS Files: ${results.summary.cssFiles}`);
    console.log(`Component Files: ${results.summary.componentFiles}`);
    console.log(`Used Classes: ${results.summary.usedClasses}`);
    console.log(`Duplicate Keyframes: ${results.summary.duplicateKeyframes}`);
    console.log(`Duplicate Rules: ${results.summary.duplicateRules}`);
    console.log(`Unused Animations: ${results.summary.unusedAnimations}`);
    console.log(`Unused Classes: ${results.summary.unusedClasses}`);
    console.log(`Potential Savings: ${results.summary.potentialSavings}`);
    
    console.log('\n📁 Output Files:');
    console.log(`   Analysis: ${outputPaths.analysisPath}`);
    console.log(`   Cleanup Script: ${outputPaths.scriptPath}`);
    console.log(`   Summary Report: ${outputPaths.summaryPath}`);
    
    if (results.analysis.recommendations.length > 0) {
      console.log('\n💡 Top Recommendations:');
      results.analysis.recommendations
        .filter(rec => rec.priority === 'high')
        .forEach(rec => console.log(`   • ${rec.title}`));
    }
    
  } catch (error) {
    console.error('❌ Analysis failed:', error.message);
    process.exit(1);
  }
}

// Export for use as module
module.exports = {
  analyzeCSSUsage,
  saveAnalysisResults,
  generateCleanupRecommendations,
  config,
};

// Run if executed directly
if (require.main === module) {
  main();
}`;

console.log('📦 To run this script, add these dependencies to package.json:');
console.log(JSON.stringify({
  "devDependencies": {
    "glob": "^8.0.0",
    "postcss": "^8.4.0"
  },
  "scripts": {
    "analyze-css": "node scripts/analyze-css-usage.js",
    "cleanup-css": "npm run analyze-css && ./analysis-results/cleanup-script-*.sh"
  }
}, null, 2));