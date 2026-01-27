# API Patterns Documentation

## Overview

This document describes the three API patterns used in this codebase and when to use each one.

---

## Pattern 1: Server-Side Cache (Direct Backend Fetch)

**Use for**: Expensive queries that benefit from caching (stock data, analytics)

```typescript
// In Server Actions or Server Components
import { createFrontendApiClient } from '@/shared/api';

export async function getExpensiveData() {
  const client = createFrontendApiClient({ serverSide: true });
  const response = await client.get('/api/stock/data', params, {
    next: { revalidate: 300 }, // Cache for 5 minutes
  });
  return response;
}
```

**Characteristics**:
- Bypasses Next.js proxy
- Connects directly to backend
- Uses Next.js `fetch` caching with `revalidate`
- Server-side only

**When to use**:
- Server Actions that fetch expensive data
- Data that changes infrequently
- Stock data, analytics, etc.

---

## Pattern 2: Direct Auth Fetch (Bypass Proxy)

**Use for**: Authentication routes that need direct backend access

```typescript
// In Server Components/Actions
import { createFrontendApiClient } from '@/shared/api';

export async function refreshTokenAction() {
  const client = createFrontendApiClient({ serverSide: true });
  const response = await client.post('/api/auth/refresh', body);
  return response;
}
```

**Characteristics**:
- Server-side only
- Direct backend connection
- No middleware/proxy interference
- Used for auth flows

**When to use**:
- Authentication endpoints
- Routes that need direct backend access
- Server Actions involving auth tokens

---

## Pattern 3: Client Proxy (Default)

**Use for**: Standard API calls from client components

```typescript
// In Client Components
import { createFrontendApiClient } from '@/shared/api';

function MyComponent() {
  const { data } = useSWR('/api/user', async () => {
    const client = createFrontendApiClient();
    return await client.get('/api/user/profile');
  });

  return <div>{data?.name}</div>;
}
```

**Characteristics**:
- Routes through `/api/proxy`
- Middleware injects auth tokens automatically
- Handles 401 → refresh → retry
- Client-side or server-side

**When to use**:
- Most client-side API calls
- Data fetching in React components
- When automatic token refresh is needed

---

## Decision Tree

```
Need to fetch data?
│
├─ Server Action/Component?
│  ├─ Yes → Is it auth-related?
│  │        ├─ Yes → Pattern 2 (Direct Auth)
│  │        └─ No  → Pattern 1 (Server Cache)
│  │
│  └─ No (Client Component) → Pattern 3 (Client Proxy)
```

---

## Quick Reference

| Pattern | Client | ServerSide | Proxy | Cache | Use Case |
|---------|--------|-------------|-------|-------|----------|
| Server Cache | ❌ | ✅ | ❌ | ✅ | Expensive data |
| Direct Auth | ❌ | ✅ | ❌ | ❌ | Auth endpoints |
| Client Proxy | ✅ | ✅ | ✅ | ❌ | Everything else |
