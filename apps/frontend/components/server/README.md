# Server Components

Server Components that run on the server during SSR. These components:
- Cannot use client-side features (useState, useEffect, event handlers)
- Have access to server-side APIs and databases
- Render HTML on the server before sending to client
- Should NOT have 'use client' directive

## Guidelines
- Keep components purely for server-side rendering
- Use Server Actions for form handling
- Fetch data directly in components
- Pass data to client components as props