# EPSX Admin Frontend

A standalone Next.js 15 application for EPSX platform administration, featuring user management, analytics, billing, and system configuration.

## Features

- **User Management**: Comprehensive user account management with role-based access control
- **IAM System**: Identity and Access Management with granular permissions
- **Analytics Dashboard**: Real-time analytics and reporting capabilities
- **Billing Management**: Subscription and payment processing administration
- **System Monitoring**: Health monitoring and performance metrics
- **Audit Logging**: Complete audit trail for administrative actions

## Tech Stack

- **Framework**: Next.js 15.4.6 with App Router
- **Runtime**: React 19.1.0 with Server Components
- **Styling**: Tailwind CSS 4.0.15 with Radix UI components
- **Authentication**: Firebase Auth + JWT tokens
- **State Management**: Zustand + SWR for server state
- **Form Handling**: React Hook Form + Zod validation
- **Testing**: Jest + Playwright + React Testing Library
- **Deployment**: Docker + Google Cloud Run

## Getting Started

### Prerequisites

- Node.js 18+ 
- npm 8+
- Docker (for containerized deployment)

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/fluke-jesadakorn/epsx-admin-frontend
   cd epsx-admin-frontend
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Set up environment variables**
   ```bash
   cp .env.example .env.local
   # Edit .env.local with your configuration
   ```

4. **Start development server**
   ```bash
   npm run dev
   ```

5. **Access the application**
   - Local: http://localhost:3001
   - Admin login required for access

### Environment Variables

```env
# Backend API Configuration
NEXT_PUBLIC_BACKEND_URL=http://localhost:8080
BACKEND_URL=http://localhost:8080

# Firebase Configuration
NEXT_PUBLIC_FIREBASE_API_KEY=your_api_key
NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=your_project.firebaseapp.com
NEXT_PUBLIC_FIREBASE_PROJECT_ID=your_project_id

# OAuth Configuration  
GOOGLE_CLIENT_ID=your_google_client_id
GOOGLE_CLIENT_SECRET=your_google_client_secret

# Application Configuration
NEXTAUTH_URL=http://localhost:3001
NEXTAUTH_SECRET=your_nextauth_secret
```

## Available Scripts

### Development
```bash
npm run dev          # Start development server on port 3001
npm run build        # Build for production
npm run start        # Start production server
```

### Code Quality
```bash
npm run lint         # Run ESLint
npm run lint:fix     # Fix ESLint issues automatically
npm run type-check   # Run TypeScript type checking
```

### Testing
```bash
npm run test         # Run Jest unit tests
npm run test:watch   # Run tests in watch mode
npm run test:e2e     # Run Playwright E2E tests
npm run test:e2e:ui  # Run E2E tests with UI
```

### Utilities
```bash
npm run analyze      # Analyze bundle size
npm run clean        # Clean build artifacts
```

## Project Structure

```
├── app/                    # Next.js App Router pages
│   ├── api/               # API routes
│   ├── login/             # Authentication pages
│   ├── users/             # User management
│   ├── analytics/         # Analytics dashboard
│   ├── billing/           # Billing management
│   └── settings/          # System settings
├── components/            # React components
│   ├── admin/            # Admin-specific components
│   ├── auth/             # Authentication components
│   ├── ui/               # Reusable UI components
│   └── guards/           # Access control guards
├── lib/                   # Utility libraries
│   ├── auth/             # Authentication utilities
│   ├── actions/          # Server actions
│   └── types/            # TypeScript types
├── hooks/                 # Custom React hooks
├── services/             # API service layer
├── config/               # Configuration files
└── __test__/             # Test files
```

## Authentication Flow

1. **Admin Login**: Firebase Auth with admin role validation
2. **JWT Tokens**: Backend issues JWT tokens with admin permissions
3. **Role Validation**: Middleware validates admin modules and permissions
4. **Session Management**: Server-side session storage with Redis

## Admin Permissions

The application uses a granular permission system:

- `admin-full-004`: Full system administration
- `user-operations`: User account management
- `analytics-specialist`: Analytics and reporting
- `billing-admin`: Billing and subscription management
- `system-admin`: System configuration and monitoring

## API Integration

The application communicates with the backend through:

- **REST APIs**: Standard CRUD operations
- **Server Actions**: Form submissions and mutations
- **WebSocket**: Real-time updates for dashboard metrics
- **OAuth 2.0**: Secure authentication flow

## Deployment

### Docker

```bash
# Build image
docker build -t epsx-admin-frontend .

# Run container
docker run -p 3001:3000 \
  -e NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io \
  epsx-admin-frontend
```

### Google Cloud Run

```bash
# Deploy to Cloud Run
gcloud run deploy epsx-admin-frontend \
  --source . \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated
```

### Environment-Specific Deployments

- **Staging**: Auto-deployed on `develop` branch
- **Production**: Auto-deployed on `main` branch
- **Feature Branches**: Manual deployment available

## Security

- **HTTPS Only**: All communications encrypted
- **CSP Headers**: Content Security Policy implemented
- **CORS**: Restricted to authorized origins
- **JWT Validation**: All API requests validated
- **Role-Based Access**: Granular permission checking
- **Audit Logging**: All admin actions logged

## Performance

- **Server Components**: Optimized rendering
- **Code Splitting**: Automatic route-based splitting
- **Image Optimization**: Next.js automatic optimization
- **Bundle Analysis**: Regular bundle size monitoring
- **Caching**: Multi-tier caching strategy

## Contributing

1. **Fork the repository**
2. **Create feature branch**: `git checkout -b feature/amazing-feature`
3. **Commit changes**: `git commit -m 'Add amazing feature'`
4. **Push to branch**: `git push origin feature/amazing-feature`
5. **Open Pull Request**

### Development Guidelines

- Follow TypeScript strict mode
- Use React Server Components where possible
- Implement proper error boundaries
- Add tests for new features
- Update documentation

## Troubleshooting

### Common Issues

**Build Failures**
```bash
# Clear cache and reinstall
npm run clean
rm -rf node_modules package-lock.json
npm install
```

**Authentication Issues**
- Verify Firebase configuration
- Check backend JWT validation
- Confirm admin role assignment

**API Connection Issues**
- Verify `NEXT_PUBLIC_BACKEND_URL` configuration
- Check network connectivity
- Confirm CORS settings

## Support

- **Documentation**: See `/docs` directory
- **Issues**: GitHub Issues
- **Security**: security@epsx.io

## License

This project is licensed under the UNLICENSED License - see the LICENSE file for details.

---

**EPSX Team** - Building the future of trading platforms