#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Define replacement mappings
const replacements = {
  // UI Components
  "import { Button } from '@/components/ui/button';": "import { Button } from '@/components/ui/button';",
  "import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';": "import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';",
  "import { Badge } from '@/components/ui/badge';": "import { Badge } from '@/components/ui/badge';",
  "import { Skeleton } from '@/components/ui/skeleton';": "import { Skeleton } from '@/components/ui/skeleton';",
  "import { Card, CardContent } from '@/components/ui/card';": "import { Card, CardContent } from '@/components/ui/card';",
  "import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';": "import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';\nimport { Badge } from '@/components/ui/badge';\nimport { Button } from '@/components/ui/button';",
  "import { Card, CardContent, CardHeader, Skeleton } from \"@epsx/ui\";": "import { Card, CardContent, CardHeader } from '@/components/ui/card';\nimport { Skeleton } from '@/components/ui/skeleton';",
  
  // Auth utilities
  "import { verifyJWT } from '@/lib/auth-utils';": "import { verifyJWT } from '@/lib/auth-utils';",
  "import { isJWTExpired, getJWTTimeToExpiry } from '@/lib/auth-utils';": "import { isJWTExpired, getJWTTimeToExpiry } from '@/lib/auth-utils';",
  
  // API Client
  "import { createApiClient, isApiError } from '@/lib/api-client';": "import { createApiClient, isApiError } from '@/lib/api-client';",
  "import { createApiClient, isApiError } from '@/lib/api-client';": "import { createApiClient, isApiError } from '@/lib/api-client';",
  "import { apiClient } from '@/lib/api-client';": "import { apiClient } from '@/lib/api-client';",
  "import type {WatchlistAddRequest as _WatchlistAddRequest, PriceAlertCreateRequest} from '@/lib/api-client';": "import type {WatchlistAddRequest as _WatchlistAddRequest, PriceAlertCreateRequest} from '@/lib/api-client';",
  "import type {PushSubscriptionRequest} from '@/lib/api-client';": "import type {PushSubscriptionRequest} from '@/lib/api-client';",
  
  // Server Actions
  "import { getCurrentUser } from '@/lib/server-actions';": "import { getCurrentUser } from '@/lib/server-actions';",
  "import { checkFeatureAccess } from '@/lib/server-actions';": "import { checkFeatureAccess } from '@/lib/server-actions';",
  "import type { PaymentStatus, PaymentTransaction as PaymentTx } from '@/lib/server-actions';": "import type { PaymentStatus, PaymentTransaction as PaymentTx } from '@/lib/server-actions';",
  
  // Theme
  "import { GlobalThemeProvider } from '@/components/providers/ThemeProvider';": "import { GlobalThemeProvider } from '@/components/providers/ThemeProvider';",
  
  // Shared Utils
  "import { fmtCurrency } from '@/lib/utils';": "import { fmtCurrency } from '@/lib/utils';"
};

function replaceInFile(filePath) {
  try {
    let content = fs.readFileSync(filePath, 'utf8');
    let changed = false;
    
    for (const [oldImport, newImport] of Object.entries(replacements)) {
      if (content.includes(oldImport)) {
        content = content.replace(new RegExp(oldImport.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'g'), newImport);
        changed = true;
      }
    }
    
    if (changed) {
      fs.writeFileSync(filePath, content, 'utf8');
      console.log(`✅ Fixed imports in: ${filePath}`);
      return true;
    }
    
    return false;
  } catch (error) {
    console.error(`❌ Error processing ${filePath}:`, error.message);
    return false;
  }
}

function walkDirectory(dir, extensions = ['.ts', '.tsx', '.js', '.jsx']) {
  const files = [];
  
  function walk(currentDir) {
    const items = fs.readdirSync(currentDir);
    
    for (const item of items) {
      const fullPath = path.join(currentDir, item);
      const stat = fs.statSync(fullPath);
      
      if (stat.isDirectory() && !['node_modules', '.next', '.git', 'dist', 'build'].includes(item)) {
        walk(fullPath);
      } else if (stat.isFile() && extensions.some(ext => item.endsWith(ext))) {
        files.push(fullPath);
      }
    }
  }
  
  walk(dir);
  return files;
}

// Main execution
const frontendDir = process.cwd();
console.log(`🔧 Processing frontend application in: ${frontendDir}`);

const files = walkDirectory(frontendDir);
let fixedCount = 0;

for (const file of files) {
  if (replaceInFile(file)) {
    fixedCount++;
  }
}

console.log(`\n🎉 Migration complete! Fixed imports in ${fixedCount} files.`);

if (fixedCount > 0) {
  console.log('\n📝 Next steps:');
  console.log('1. Run "npm run type-check" to verify TypeScript compilation');
  console.log('2. Run "npm run build" to test the build process');
  console.log('3. Fix any remaining import issues manually');
}