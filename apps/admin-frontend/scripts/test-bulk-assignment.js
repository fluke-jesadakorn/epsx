// Test script to verify bulk user level assignment functionality
// Run this in your browser console on the admin users page

async function testBulkUserLevelAssignment() {
  console.log('Testing bulk user level assignment...');
  
  try {
    // Test with a non-existent email
    const testData = {
      updates: [
        {
          uid: 'test@example.com',
          userLevel: 'SILVER',
          reason: 'Test assignment'
        }
      ]
    };
    
    const response = await fetch('/api/admin/users/bulk-update-levels', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(testData),
    });
    
    const result = await response.json();
    console.log('Test result:', result);
    
    if (result.results && result.results.length > 0) {
      const firstResult = result.results[0];
      if (firstResult.success) {
        console.log('✅ User level assignment successful');
      } else {
        console.log('❌ User level assignment failed:', firstResult.error);
      }
    }
    
  } catch (error) {
    console.error('Test failed:', error);
  }
}

// Uncomment the line below to run the test
// testBulkUserLevelAssignment();

console.log('Bulk user level assignment test script loaded. Call testBulkUserLevelAssignment() to run the test.');
