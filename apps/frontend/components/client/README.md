# Client Components

Client Components that run in the browser with 'use client' directive. These components:
- Can use client-side features (useState, useEffect, event handlers)
- Handle user interactions and dynamic behavior
- Require client-side JavaScript execution
- MUST have 'use client' directive at the top

## Guidelines
- Add 'use client' directive at the top of files
- Use for interactive components only
- Keep minimal to reduce bundle size
- Receive data from Server Components as props