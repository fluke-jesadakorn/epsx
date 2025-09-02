# EPSX Admin Frontend

Next.js 15 admin dashboard for EPSX platform with OIDC authentication and structured permissions.

## Features

- **User Management**: Complete user administration with structured permissions
- **Analytics Dashboard**: Real-time analytics and reporting
- **System Monitoring**: Health monitoring and performance metrics
- **Audit Logging**: Complete audit trail for administrative actions
- **OIDC Authentication**: OpenID Connect compliant authentication with Bearer tokens
- **Structured Permissions**: Platform-scoped permission system (admin:*:*)

## Tech Stack

- **Framework**: Next.js 15.5.0 with App Router
- **Runtime**: React 19.1.0 with Server Components
- **Styling**: Tailwind CSS 4.0.15 with Radix UI
- **Authentication**: OIDC with Firebase + Bearer tokens
- **State**: Zustand + SWR
- **Forms**: React Hook Form + Zod
- **Testing**: Jest + Playwright
- **Deployment**: Docker + Cloud Run

## Quick Start

```bash
# Install dependencies
pnpm install

# Start development
pnpm dev:admin  # Port 3001

# Build
pnpm build:admin
```

## Environment Variables

```env
# Backend API
NEXT_PUBLIC_BACKEND_URL=http://localhost:8080
NEXT_PUBLIC_API_URL=http://localhost:8080

# Admin URLs
NEXT_PUBLIC_ADMIN_URL=http://localhost:3001

# OIDC Configuration
NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-admin
NEXT_PUBLIC_OAUTH_AUTHORIZATION_ENDPOINT=http://localhost:8080/oauth/authorize
NEXT_PUBLIC_OAUTH_TOKEN_ENDPOINT=http://localhost:8080/oauth/token
NEXT_PUBLIC_OAUTH_USERINFO_ENDPOINT=http://localhost:8080/oauth/userinfo

# Firebase (for initial auth)
NEXT_PUBLIC_FIREBASE_API_KEY=your_key
NEXT_PUBLIC_FIREBASE_PROJECT_ID=your_project
NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=your_domain
```

## Authentication Flow

1. **Firebase Login**: Initial authentication via Firebase Auth
2. **OIDC Exchange**: Exchange Firebase token for OIDC tokens
3. **Bearer Tokens**: Backend validates Bearer tokens with RS256 JWT
4. **Structured Permissions**: Validate admin permissions (admin:*:*)
5. **HttpOnly Cookies**: Store OIDC tokens securely

## Permission System

The application uses structured permissions with format `"platform:resource:action"`:

- `admin:*:*` - Full admin access
- `admin:users:manage` - User management
- `admin:analytics:view` - Analytics access
- `admin:system:configure` - System settings

## Project Structure

```
├── app/
│   ├── api/auth/           # OIDC callback handlers
│   ├── dashboard/          # Admin dashboard
│   ├── users/              # User management
│   ├── analytics/          # Analytics pages
│   └── settings/           # System settings
├── components/
│   ├── admin/              # Admin components
│   ├── auth/               # Auth components
│   └── ui/                 # UI components
├── lib/
│   ├── admin-oidc-auth.ts  # OIDC utilities
│   ├── admin-client.ts     # API client
│   └── admin-types.ts      # TypeScript types
└── middleware.ts           # Route protection
```

## API Integration

- **REST APIs**: Standard CRUD with Bearer token auth
- **Server Actions**: Form submissions
- **OIDC Endpoints**: Token exchange and validation
- **Session Management**: HttpOnly cookie storage

## Deployment

### Docker
```bash
docker build -t epsx-admin-frontend .
docker run -p 3001:3000 epsx-admin-frontend
```

### Cloud Run
```bash
gcloud run deploy epsx-admin-frontend \
  --source . \
  --platform managed \
  --region us-central1
```

## Security

- **OIDC Compliant**: OpenID Connect standard authentication
- **HTTPS Only**: All communications encrypted
- **Bearer Tokens**: RS256 JWT validation
- **HttpOnly Cookies**: Secure token storage
- **CORS**: Restricted origins
- **CSP Headers**: Content security policies

## Performance

- **Server Components**: Optimized SSR
- **Code Splitting**: Route-based splitting
- **Caching**: Multi-tier caching
- **Bundle Analysis**: Size monitoring

## Development

```bash
# Development server
pnpm dev:admin

# Type checking
pnpm type-check

# Linting
pnpm lint
pnpm lint:fix

# Testing
pnpm test
pnpm test:e2e
```

## Troubleshooting

**OIDC Authentication Issues:**
- Check OAuth client configuration
- Verify callback URL matches registration
- Ensure Bearer token validation works

**Permission Issues:**
- Confirm user has admin:*:* permissions
- Check structured permission format
- Verify backend permission validation

**Build Issues:**
```bash
pnpm clean
rm -rf node_modules
pnpm install
```