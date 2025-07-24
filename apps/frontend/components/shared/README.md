# Shared Components

Isomorphic components that can work in both server and client environments. These components:
- Are pure functional components without client-side features
- Can be imported and used by both Server and Client Components
- Should not have 'use client' directive
- Work with SSR and client hydration

## Guidelines
- Keep components stateless and pure
- No client-side hooks or event handlers
- Can be used in both server and client contexts
- Focus on presentation and layout