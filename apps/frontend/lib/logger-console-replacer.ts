/**
 * Console Logger Replacer
 * Systematically replaces console.log statements with proper logging
 * Addresses the critical code smell of 1000+ console statements in production
 */

import { logger, apiLogger, authLogger, devLog, safeError } from '@/lib/logger';

interface ConsoleReplacementRule {
  pattern: RegExp;
  replacement: string;
  category: 'error' | 'warn' | 'info' | 'debug';
  context?: string;
}

/**
 * Replacement rules for different console statement patterns
 */
export const CONSOLE_REPLACEMENT_RULES: ConsoleReplacementRule[] = [
  // Error patterns - replace with proper error logging
  {
    pattern: /console\.error\(['"`]Error ([^'"`]+):['"`],\s*error\)/g,
    replacement: 'logger.error(\'$1 failed\', safeError(error))',
    category: 'error'
  },
  {
    pattern: /console\.error\(['"`]([^'"`]+)['"`],\s*error\)/g,
    replacement: 'logger.error(\'$1\', safeError(error))',
    category: 'error'
  },
  {
    pattern: /console\.error\(['"`]([^'"`]+)['"`]\)/g,
    replacement: 'logger.error(\'$1\')',
    category: 'error'
  },
  
  // API-related logging - use apiLogger
  {
    pattern: /console\.error\(['"`].*API.*['"`],\s*([^)]+)\)/g,
    replacement: 'apiLogger.error(\'API request failed\', safeError($1))',
    category: 'error',
    context: 'api'
  },
  {
    pattern: /console\.error\(['"`].*request.*['"`],\s*([^)]+)\)/gi,
    replacement: 'apiLogger.error(\'Request failed\', safeError($1))',
    category: 'error',
    context: 'api'
  },
  
  // Authentication-related logging - use authLogger
  {
    pattern: /console\.error\(['"`].*auth.*['"`],\s*([^)]+)\)/gi,
    replacement: 'authLogger.error(\'Authentication failed\', safeError($1))',
    category: 'error',
    context: 'auth'
  },
  {
    pattern: /console\.error\(['"`].*token.*['"`],\s*([^)]+)\)/gi,
    replacement: 'authLogger.error(\'Token operation failed\', safeError($1))',
    category: 'error',
    context: 'auth'
  },
  
  // Warning patterns
  {
    pattern: /console\.warn\(['"`]([^'"`]+)['"`],?\s*([^)]*)\)/g,
    replacement: 'logger.warn(\'$1\'$2)',
    category: 'warn'
  },
  
  // Info patterns  
  {
    pattern: /console\.info\(['"`]([^'"`]+)['"`],?\s*([^)]*)\)/g,
    replacement: 'logger.info(\'$1\'$2)',
    category: 'info'
  },
  
  // Debug patterns - use devLog for development-only logging
  {
    pattern: /console\.log\(['"`]([^'"`]+)['"`],?\s*([^)]*)\)/g,
    replacement: 'devLog(\'$1\'$2)',
    category: 'debug'
  },
  {
    pattern: /console\.debug\(['"`]([^'"`]+)['"`],?\s*([^)]*)\)/g,
    replacement: 'devLog(\'$1\'$2)',
    category: 'debug'
  }
];

/**
 * Context-specific logger mappings
 */
export const CONTEXT_LOGGERS = {
  api: 'apiLogger',
  auth: 'authLogger',
  default: 'logger'
};

/**
 * Get the appropriate logger import for a context
 */
export function getLoggerImports(contexts: Set<string>): string {
  const imports = ['logger', 'devLog', 'safeError'];
  
  if (contexts.has('api')) imports.push('apiLogger');
  if (contexts.has('auth')) imports.push('authLogger');
  
  return `import { ${imports.join(', ')} } from '@/lib/logger';`;
}

/**
 * Apply console replacement rules to content
 */
export function replaceConsoleStatements(content: string): {
  updatedContent: string;
  replacementCount: number;
  contexts: Set<string>;
} {
  let updatedContent = content;
  let replacementCount = 0;
  const contexts = new Set<string>();
  
  for (const rule of CONSOLE_REPLACEMENT_RULES) {
    const matches = updatedContent.match(rule.pattern);
    if (matches) {
      replacementCount += matches.length;
      if (rule.context) {
        contexts.add(rule.context);
      }
      updatedContent = updatedContent.replace(rule.pattern, rule.replacement);
    }
  }
  
  return {
    updatedContent,
    replacementCount,
    contexts
  };
}

/**
 * Process a file to replace console statements
 */
export function processFileConsoleStatements(filePath: string, content: string): {
  updatedContent: string;
  needsLoggerImport: boolean;
  loggerImports: string;
  replacementCount: number;
} {
  const { updatedContent, replacementCount, contexts } = replaceConsoleStatements(content);
  
  const needsLoggerImport = replacementCount > 0;
  const loggerImports = needsLoggerImport ? getLoggerImports(contexts) : '';
  
  // Add import if needed and not already present
  let finalContent = updatedContent;
  if (needsLoggerImport && !content.includes("from '@/lib/logger'")) {
    // Insert import after existing imports or at the top
    const importInsertPosition = findImportInsertPosition(content);
    finalContent = 
      content.slice(0, importInsertPosition) +
      loggerImports + '\n' +
      content.slice(importInsertPosition);
    
    // Apply replacements after import insertion
    finalContent = replaceConsoleStatements(finalContent).updatedContent;
  }
  
  return {
    updatedContent: finalContent,
    needsLoggerImport,
    loggerImports,
    replacementCount
  };
}

/**
 * Find the best position to insert logger imports
 */
function findImportInsertPosition(content: string): number {
  const lines = content.split('\n');
  let lastImportIndex = -1;
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    if (line.startsWith('import ') || line.startsWith('const ') && line.includes(' = require(')) {
      lastImportIndex = i;
    } else if (line && !line.startsWith('//') && !line.startsWith('/*')) {
      break;
    }
  }
  
  if (lastImportIndex >= 0) {
    // Insert after last import
    const lineEnding = content.indexOf('\n', content.indexOf('\n', content.split('\n').slice(0, lastImportIndex + 1).join('\n').length));
    return lineEnding >= 0 ? lineEnding + 1 : content.length;
  }
  
  return 0; // Insert at beginning if no imports found
}

/**
 * Generate report of console replacements
 */
export function generateReplacementReport(results: Array<{ filePath: string; replacementCount: number }>): string {
  const totalReplacements = results.reduce((sum, r) => sum + r.replacementCount, 0);
  const filesModified = results.filter(r => r.replacementCount > 0).length;
  
  const report = [
    '# Console Statement Replacement Report',
    '',
    `## Summary`,
    `- Total console statements replaced: **${totalReplacements}**`,
    `- Files modified: **${filesModified}**`,
    `- Production readiness improvement: **Critical**`,
    '',
    '## Files Modified',
  ];
  
  results
    .filter(r => r.replacementCount > 0)
    .sort((a, b) => b.replacementCount - a.replacementCount)
    .forEach(({ filePath, replacementCount }) => {
      report.push(`- \`${filePath}\`: ${replacementCount} replacements`);
    });
  
  report.push('');
  report.push('## Benefits');
  report.push('- ✅ Production-ready logging system');
  report.push('- ✅ Proper error tracking and debugging');
  report.push('- ✅ Development vs production log level control');
  report.push('- ✅ Structured logging with context');
  report.push('- ✅ No console pollution in production');
  
  return report.join('\n');
}