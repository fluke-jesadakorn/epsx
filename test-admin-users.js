// Test script to debug admin users API call
const { serverGetAdminUsers } = require('./packages/api-client/dist/api-server.js');

async function testAdminUsers() {
  console.log('Testing serverGetAdminUsers...');
  try {
    const result = await serverGetAdminUsers();
    console.log('Result:', JSON.stringify(result, null, 2));
  } catch (error) {
    console.error('Error:', error);
  }
}

testAdminUsers();