# Admin Frontend E2E Tests

This directory contains end-to-end tests specifically for the admin frontend application.

## Test Structure

- `/admin/` - Contains admin-specific test suites
- `/helpers/` - Contains shared test utilities and helper functions

## Test Files

### admin-authorization.spec.ts
Tests for admin authorization and access control:
- Admin dashboard access
- User management access  
- Admin API access
- Admin middleware functionality

## Running Tests

From the admin-frontend directory:

```bash
# Run all tests
pnpm test

# Run tests with UI
pnpm test:ui

# Run tests in headed mode (visible browser)
pnpm test:headed
```

## Configuration

Tests are configured to run against the admin frontend at `http://localhost:3001`.

Make sure the admin frontend application is running before executing tests:

```bash
pnpm dev:admin
```

## Test Data

Tests use the shared test helpers from `/helpers/test-users.ts` to create and manage test user accounts during test execution.
