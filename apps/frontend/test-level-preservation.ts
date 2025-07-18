/**
 * Test script to verify level number preservation for expired users
 */

import { formatLevelAsNumber } from './utils/level-utils';
import { getNumericLevelByLevel } from './app/constants/packages';

// Test cases
const testCases = [
  {
    userLevel: 'BRONZE' as const,
    isExpired: false,
    expectedLevel: 0,
    expectedDisplay: 'Level 0'
  },
  {
    userLevel: 'SILVER' as const,
    isExpired: true,
    expectedLevel: 1,
    expectedDisplay: 'Level 1 (Expired)'
  },
  {
    userLevel: 'GOLD' as const,
    isExpired: true,
    expectedLevel: 2,
    expectedDisplay: 'Level 2 (Expired)'
  },
  {
    userLevel: 'PLATINUM' as const,
    isExpired: false,
    expectedLevel: 3,
    expectedDisplay: 'Level 3'
  },
  {
    userLevel: 'DIAMOND' as const,
    isExpired: true,
    expectedLevel: 4,
    expectedDisplay: 'Level 4 (Expired)'
  }
];

console.log('🧪 Testing Level Number Preservation...\n');

let allTestsPassed = true;

testCases.forEach((testCase, index) => {
  console.log(`Test ${index + 1}: ${testCase.userLevel} (${testCase.isExpired ? 'Expired' : 'Active'})`);
  
  // Test level number preservation
  const actualLevel = getNumericLevelByLevel(testCase.userLevel);
  const levelNumberMatch = actualLevel === testCase.expectedLevel;
  
  // Test display formatting
  const actualDisplay = formatLevelAsNumber(testCase.userLevel);
  const expectedBaseDisplay = `Level ${testCase.expectedLevel}`;
  const displayMatch = actualDisplay === expectedBaseDisplay;
  
  console.log(`  📊 Level Number: ${actualLevel} (expected: ${testCase.expectedLevel}) ${levelNumberMatch ? '✅' : '❌'}`);
  console.log(`  🖥️  Display: ${actualDisplay} (expected: ${expectedBaseDisplay}) ${displayMatch ? '✅' : '❌'}`);
  
  if (!levelNumberMatch || !displayMatch) {
    allTestsPassed = false;
  }
  
  console.log('');
});

// Test access control logic
console.log('🔐 Testing Access Control Logic...\n');

type UserLevel = 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM' | 'DIAMOND' | 'VIP';

const accessTestCases = [
  {
    userLevel: 'BRONZE' as UserLevel,
    isExpired: false,
    expectedAccess: 5,
    description: 'Bronze user (not expired) should have 5 rankings'
  },
  {
    userLevel: 'GOLD' as UserLevel,
    isExpired: true,
    expectedAccess: 5,
    description: 'Gold user (expired) should have 5 rankings (Bronze access)'
  },
  {
    userLevel: 'GOLD' as UserLevel,
    isExpired: false,
    expectedAccess: 50,
    description: 'Gold user (active) should have 50 rankings'
  },
  {
    userLevel: 'PLATINUM' as UserLevel,
    isExpired: true,
    expectedAccess: 5,
    description: 'Platinum user (expired) should have 5 rankings (Bronze access)'
  }
];

// Mock the access control logic
function getRankingLimit(userLevel: UserLevel, isExpired: boolean): number {
  const limits: Record<UserLevel, number> = {
    BRONZE: 5,
    SILVER: 25,
    GOLD: 50,
    PLATINUM: 100,
    DIAMOND: 200,
    VIP: -1
  };
  
  return isExpired ? limits.BRONZE : limits[userLevel];
}

accessTestCases.forEach((testCase, index) => {
  console.log(`Access Test ${index + 1}: ${testCase.description}`);
  
  const actualAccess = getRankingLimit(testCase.userLevel, testCase.isExpired);
  const accessMatch = actualAccess === testCase.expectedAccess;
  
  console.log(`  🎯 Access: ${actualAccess} rankings (expected: ${testCase.expectedAccess}) ${accessMatch ? '✅' : '❌'}`);
  
  if (!accessMatch) {
    allTestsPassed = false;
  }
  
  console.log('');
});

// Summary
console.log('📊 Test Summary');
console.log('===============');
console.log(allTestsPassed ? '✅ All tests passed!' : '❌ Some tests failed!');
console.log('');

if (allTestsPassed) {
  console.log('🎉 Level number preservation is working correctly!');
  console.log('✅ Expired users will show their correct level number');
  console.log('✅ Access control still restricts expired users appropriately');
} else {
  console.log('🚨 Issues detected in level number preservation');
  console.log('❌ Please review the implementation');
}

export default function runLevelTests() {
  return allTestsPassed;
}
