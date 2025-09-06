#!/usr/bin/env node

// Test script to debug admin API calls
const http = require('http');

// Get JWT token from admin frontend (simulate getting it from cookie)
// In real scenario, this would come from server-side cookie
const TEST_JWT = process.env.TEST_JWT || '';

console.log('Testing admin API with JWT:', TEST_JWT ? 'Present' : 'Missing');

const options = {
  hostname: 'localhost',
  port: 8080,
  path: '/api/v1/admin/users?offset=0&limit=5',
  method: 'GET',
  headers: {
    'Content-Type': 'application/json',
    ...(TEST_JWT && { Authorization: `Bearer ${TEST_JWT}` }),
  },
};

const req = http.request(options, res => {
  console.log(`Status: ${res.statusCode}`);
  console.log(`Headers:`, res.headers);

  let data = '';
  res.on('data', chunk => {
    data += chunk;
  });

  res.on('end', () => {
    console.log('Response:', data || 'Empty');
  });
});

req.on('error', e => {
  console.error(`Problem with request: ${e.message}`);
});

req.end();
