# EPSX Holding Company Admin Frontend

A standalone Next.js 15 application for EPSX Holding Company administration, featuring multi-entity management, cross-entity analytics, consolidated user management, and entity-scoped system configuration across EPSX Platform, EPSX Pay, and EPSX Token.

## Features

### Core Holding Features
- **Multi-Entity Management**: Manage EPSX Platform, EPSX Pay, and EPSX Token from unified interface
- **Cross-Entity Analytics**: Consolidated reporting and analytics across all business units
- **Entity-Scoped Permissions**: Role-based access control with entity-level isolation
- **Consolidated User Management**: Unified user administration across all EPSX entities
- **Holding-Level Compliance**: Group-wide compliance monitoring and audit trails

### Entity-Specific Features
- **EPSX Platform**: Trading analytics, stock rankings, TradingView integration
- **EPSX Pay**: Payment processing, transaction monitoring, KYC/AML compliance
- **EPSX Token**: Token management, smart contracts, tokenomics analytics

### Traditional Admin Features
- **User Management**: Comprehensive user account management with role-based access control
- **IAM System**: Identity and Access Management with granular permissions
- **Analytics Dashboard**: Real-time analytics and reporting capabilities
- **Billing Management**: Subscription and payment processing administration
- **System Monitoring**: Health monitoring and performance metrics
- **Audit Logging**: Complete audit trail for administrative actions

## Holding Company Architecture

### Business Structure
```
EPSX Holding Company (Parent Entity)
├── EPSX Platform (Trading & Analytics)
│   ├── Stock Rankings & EPS Analytics
│   ├── TradingView Integration
│   ├── Real-time Market Data
│   └── Trading Dashboard
├── EPSX Pay (Payment Processing)
│   ├── Transaction Processing
│   ├── KYC/AML Compliance
│   ├── Payment Analytics
│   └── Fraud Detection
└── EPSX Token (Cryptocurrency Services)
    ├── Token Management
    ├── Smart Contract Administration
    ├── Tokenomics Analytics
    └── DeFi Integration
```

### Multi-Entity Admin Design
- **Unified Interface**: Single admin portal managing all entities
- **Entity Isolation**: Data and permissions scoped by entity
- **Cross-Entity Features**: Consolidated analytics and user management
- **Role-Based Access**: Granular permissions across entity boundaries

## Tech Stack

- **Framework**: Next.js 15.5.0 with App Router
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

# Entity-Specific Backend URLs (Optional - defaults to main backend)
NEXT_PUBLIC_PLATFORM_API_URL=http://localhost:8080/platform
NEXT_PUBLIC_PAY_API_URL=http://localhost:8080/pay  
NEXT_PUBLIC_TOKEN_API_URL=http://localhost:8080/token
NEXT_PUBLIC_HOLDING_API_URL=http://localhost:8080/holding

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

# Entity Configuration
NEXT_PUBLIC_DEFAULT_ENTITY=holding  # Default entity view on login
NEXT_PUBLIC_ENABLE_ENTITY_SWITCHING=true  # Allow switching between entities
NEXT_PUBLIC_ENTITIES=platform,pay,token,holding  # Available entities

# Feature Flags
NEXT_PUBLIC_ENABLE_CROSS_ENTITY_ANALYTICS=true
NEXT_PUBLIC_ENABLE_HOLDING_COMPLIANCE=true
NEXT_PUBLIC_ENABLE_ENTITY_ISOLATION=true
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
│   ├── holding/           # Holding company dashboard & cross-entity features
│   │   ├── dashboard/     # Consolidated holding dashboard
│   │   ├── users/         # Cross-entity user management
│   │   ├── analytics/     # Cross-entity analytics
│   │   └── compliance/    # Holding-level compliance
│   ├── platform/          # EPSX Platform entity admin
│   │   ├── dashboard/     # Platform-specific dashboard
│   │   ├── users/         # Platform user management
│   │   ├── analytics/     # Trading & stock analytics
│   │   └── rankings/      # Stock rankings management
│   ├── pay/               # EPSX Pay entity admin
│   │   ├── dashboard/     # Payment dashboard
│   │   ├── users/         # Payment user management
│   │   ├── transactions/  # Transaction monitoring
│   │   └── compliance/    # KYC/AML compliance
│   ├── token/             # EPSX Token entity admin
│   │   ├── dashboard/     # Token dashboard
│   │   ├── users/         # Token holder management
│   │   ├── contracts/     # Smart contract admin
│   │   └── analytics/     # Tokenomics analytics
│   ├── login/             # Authentication pages
│   └── settings/          # System settings
├── components/            # React components
│   ├── admin/            # Admin-specific components
│   ├── auth/             # Authentication components
│   ├── ui/               # Reusable UI components
│   ├── guards/           # Access control guards
│   ├── holding/          # Holding company components
│   ├── platform/         # Platform-specific components
│   ├── pay/              # Payment-specific components
│   └── token/            # Token-specific components
├── lib/                   # Utility libraries
│   ├── auth/             # Authentication utilities
│   ├── actions/          # Server actions
│   ├── types/            # TypeScript types
│   └── entities/         # Entity-specific utilities
├── hooks/                 # Custom React hooks
├── services/             # API service layer
│   ├── holding.ts        # Holding company API service
│   ├── platform.ts       # Platform API service
│   ├── pay.ts            # Payment API service
│   └── token.ts          # Token API service
├── config/               # Configuration files
└── __test__/             # Test files
```

## Authentication Flow

1. **Admin Login**: Firebase Auth with admin role validation
2. **JWT Tokens**: Backend issues JWT tokens with admin permissions
3. **Role Validation**: Middleware validates admin modules and permissions
4. **Session Management**: Server-side session storage with Redis

## Multi-Level Permission System

### Entity-Based Role Hierarchy
The application uses a multi-tier permission system supporting the holding company structure:

#### **Holding-Level Roles**
- `holding:admin`: Full access to all entities and holding-level features
- `holding:analyst`: Cross-entity analytics and reporting access
- `holding:compliance`: Holding-level compliance and audit access

#### **Entity-Specific Roles**
- `platform:admin`: Full EPSX Platform administration
- `platform:analyst`: Platform analytics and trading data access
- `pay:admin`: Full EPSX Pay administration  
- `pay:compliance`: Payment compliance and KYC management
- `token:admin`: Full EPSX Token administration
- `token:analyst`: Token analytics and smart contract access

#### **Cross-Entity Permissions**
- `cross:user_management`: Manage users across all entities
- `cross:analytics`: Access consolidated analytics
- `cross:compliance`: Cross-entity compliance monitoring
- `cross:security`: Security management across all entities

### Permission Structure
```
Entity Permission Format: {entity}:{resource}:{action}
- platform:users:read     # Read platform users
- pay:transactions:write  # Manage payment transactions  
- token:contracts:admin   # Administer smart contracts
- holding:*:*            # Full holding access
```

### Legacy Permissions (Maintained for Compatibility)
- `admin-full-004`: Full system administration
- `user-operations`: User account management
- `analytics-specialist`: Analytics and reporting
- `billing-admin`: Billing and subscription management
- `system-admin`: System configuration and monitoring

## Navigation Structure

### Entity Selector Interface
The admin interface features a top-level entity selector allowing admins to switch between:

- **Holding Dashboard**: Consolidated view of all entities with cross-entity analytics
- **Platform Admin**: EPSX Platform-specific administration (trading, analytics, rankings)
- **Pay Admin**: EPSX Pay-specific administration (payments, KYC, transactions)
- **Token Admin**: EPSX Token-specific administration (tokens, contracts, DeFi)

### Nested Entity Features
Each entity contains standard admin modules with entity-scoped functionality:

#### **Holding Level** (`/holding/*`)
- `/holding/dashboard` - Consolidated holding company overview
- `/holding/users` - Cross-entity user management
- `/holding/analytics` - Cross-entity analytics and reporting
- `/holding/compliance` - Group-level compliance monitoring

#### **Platform Level** (`/platform/*`) 
- `/platform/dashboard` - Trading platform metrics
- `/platform/users` - Platform user management
- `/platform/analytics` - Stock rankings and EPS analytics  
- `/platform/rankings` - TradingView data management

#### **Pay Level** (`/pay/*`)
- `/pay/dashboard` - Payment processing overview
- `/pay/users` - Payment user management
- `/pay/transactions` - Transaction monitoring and management
- `/pay/compliance` - KYC/AML compliance administration

#### **Token Level** (`/token/*`)
- `/token/dashboard` - Token ecosystem overview
- `/token/users` - Token holder management
- `/token/contracts` - Smart contract administration
- `/token/analytics` - Tokenomics and DeFi analytics

### Permission-Based Navigation
Navigation items are dynamically filtered based on user permissions, ensuring admins only see features they have access to manage.

## API Integration

The application communicates with the backend through:

- **REST APIs**: Standard CRUD operations with entity-scoped endpoints
- **Server Actions**: Form submissions and mutations
- **WebSocket**: Real-time updates for dashboard metrics across entities
- **OAuth 2.0**: Secure authentication flow
- **Entity APIs**: Dedicated API endpoints for each business entity

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

## Implementation Phases

### Phase 1: Core Infrastructure (Weeks 1-2)
**Goal**: Establish multi-entity foundation and routing structure

#### Tasks:
- [ ] Create entity-based routing structure (`/holding/*`, `/platform/*`, `/pay/*`, `/token/*`)
- [ ] Implement multi-tenant permission system with entity-scoped roles
- [ ] Add entity context throughout admin application
- [ ] Create entity selector/switcher component
- [ ] Establish entity isolation in data fetching and state management
- [ ] Update authentication flow to support entity-based permissions

#### Deliverables:
- Multi-entity routing structure
- Entity permission system
- Entity selector interface
- Updated authentication with entity context

### Phase 2: Entity-Specific Features (Weeks 3-5)
**Goal**: Implement dedicated admin features for each business entity

#### EPSX Pay Admin Features:
- [ ] Payment dashboard with transaction metrics
- [ ] Transaction monitoring and management interface
- [ ] KYC/AML compliance administration
- [ ] Payment user management with verification workflows
- [ ] Fraud detection and prevention tools

#### EPSX Token Admin Features:
- [ ] Token ecosystem dashboard with tokenomics metrics
- [ ] Smart contract administration interface
- [ ] Token holder management and analytics
- [ ] DeFi integration monitoring
- [ ] Token distribution and governance tools

#### EPSX Platform Enhancements:
- [ ] Migrate existing features under Platform entity
- [ ] Enhance stock rankings management
- [ ] Improve TradingView data integration
- [ ] Optimize EPS analytics dashboard

#### Deliverables:
- Complete EPSX Pay admin interface
- Complete EPSX Token admin interface  
- Enhanced EPSX Platform admin features
- Entity-specific user management

### Phase 3: Cross-Entity Features (Weeks 6-8)
**Goal**: Implement holding-level consolidated features and analytics

#### Holding Dashboard:
- [ ] Consolidated metrics across all entities
- [ ] Cross-entity analytics and reporting
- [ ] Unified user management across entities
- [ ] Holding-level compliance monitoring
- [ ] Executive summary and KPI dashboard

#### Advanced Features:
- [ ] Cross-entity user migration tools
- [ ] Consolidated audit logging and compliance reporting
- [ ] Multi-entity notification system
- [ ] Cross-entity security monitoring
- [ ] Holding-level financial analytics

#### Performance & Optimization:
- [ ] Entity-scoped caching strategies
- [ ] Cross-entity data synchronization
- [ ] Performance monitoring per entity
- [ ] Load balancing for entity-specific traffic

#### Deliverables:
- Complete holding dashboard
- Cross-entity analytics system
- Unified compliance framework
- Performance-optimized multi-entity architecture

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