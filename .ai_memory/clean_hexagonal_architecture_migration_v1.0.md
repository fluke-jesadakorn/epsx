# Clean + Hexagonal Architecture Migration Plan for EPSX
**Version:** 1.0  
**Date:** 2025-07-22  
**Status:** Planning Phase

## Executive Summary

This document outlines the migration strategy for transforming the EPSX monorepo from its current layered architecture to a **Clean Architecture combined with Hexagonal Architecture** pattern. The goal is to achieve better separation of concerns, testability, and maintainability while preserving the existing functionality.

## Current Architecture Analysis

### Current Structure Issues
- **Business Logic Dispersion**: Logic scattered across API routes, services, lib, and components
- **Mixed Data Access**: Direct Firebase calls mixed with API layer usage  
- **Inconsistent Service Patterns**: Some features use services, others use direct lib calls
- **Monolithic Components**: Page components contain business logic
- **Tight Coupling**: Direct dependencies between layers

### Current Strengths to Preserve
- Strong TypeScript type safety with shared packages
- Robust Firebase + IAM authentication system
- Scalable Turborepo build system
- Component reusability with shared UI package
- Real-time capabilities (WebSocket, Firebase listeners)

## Target Architecture: Clean + Hexagonal

### Core Principles
1. **Dependency Inversion**: Inner layers don't depend on outer layers
2. **Use Cases First**: Business logic drives the architecture
3. **Port & Adapter Pattern**: Abstract external dependencies
4. **Testable Core**: Business logic isolated and testable
5. **Framework Independence**: Core logic independent of Next.js/React

### Architecture Layers (Inside → Outside)

#### 1. **Domain Layer** (Core)
- **Entities**: Business objects with rules (User, Payment, Stock, Role, Permission)
- **Value Objects**: Immutable objects (Email, Currency, Price, DateRange)
- **Domain Services**: Domain-specific business logic
- **Domain Events**: For decoupled communication

#### 2. **Application Layer** (Use Cases)
- **Use Cases**: Business workflows (CreateUser, ProcessPayment, AuthenticateUser)
- **Interfaces**: Ports for external dependencies
- **DTOs**: Data Transfer Objects for use case boundaries
- **Application Services**: Orchestrate use cases

#### 3. **Infrastructure Layer** (Adapters)
- **Database Adapters**: MongoDB, Firestore implementations
- **External Service Adapters**: Firebase Auth, Payment gateways
- **Web Adapters**: Next.js API routes, WebSocket handlers
- **UI Adapters**: React components, hooks

#### 4. **Presentation Layer** (UI)
- **Pages**: Next.js pages and layouts
- **Components**: UI components calling use cases
- **Presenters**: Format data for UI consumption

## Proposed New Structure

### Package Restructure

```
packages/
├── @epsx/domain/              # NEW - Core business logic
│   ├── entities/              # Business entities
│   ├── value-objects/         # Immutable value objects  
│   ├── services/              # Domain services
│   ├── events/                # Domain events
│   └── specifications/        # Business rules
│
├── @epsx/application/         # NEW - Use cases layer
│   ├── use-cases/             # Business workflows
│   ├── ports/                 # Interfaces for external deps
│   ├── dtos/                  # Data transfer objects
│   └── services/              # Application services
│
├── @epsx/infrastructure/      # NEW - Adapters implementation
│   ├── database/              # DB adapters (MongoDB, Firestore)
│   ├── external-services/     # External API adapters
│   ├── auth/                  # Auth implementation adapters
│   ├── payment/               # Payment gateway adapters
│   └── messaging/             # Event handling, WebSocket
│
├── @epsx/presentation/        # NEW - UI/API presentation layer  
│   ├── web-api/               # Next.js API route handlers
│   ├── react-ui/              # React-specific UI logic
│   ├── hooks/                 # Data fetching hooks
│   └── presenters/            # Data formatting for UI
│
├── @epsx/shared-kernel/       # REFACTORED - Cross-cutting concerns
│   ├── types/                 # Common types and interfaces
│   ├── utils/                 # Pure utility functions
│   ├── errors/                # Error definitions
│   └── events/                # Shared event definitions
│
└── @epsx/ui/                  # EXISTING - UI components (unchanged)
```

### Application Restructure

```
apps/
├── frontend/
│   ├── app/                   # Next.js routes (thin controllers)
│   ├── components/            # Pure UI components
│   ├── providers/             # Dependency injection setup
│   └── config/                # App-specific configuration
│
├── admin-frontend/
│   ├── app/                   # Admin routes (thin controllers)  
│   ├── components/            # Admin UI components
│   ├── providers/             # Admin DI setup
│   └── config/                # Admin configuration
│
└── backend/                   # Rust backend (separate clean architecture)
```

## Migration Strategy

### Phase 1: Foundation Setup (Week 1-2)
#### 1.1 Create New Package Structure
- [ ] Create `@epsx/domain` package with basic entities
- [ ] Create `@epsx/application` package with use case interfaces  
- [ ] Create `@epsx/infrastructure` package with adapter interfaces
- [ ] Create `@epsx/presentation` package for UI logic
- [ ] Refactor `@epsx/shared` to `@epsx/shared-kernel`

#### 1.2 Define Core Domain Entities
- [ ] **User Entity**: Profile, preferences, subscription status
- [ ] **Role Entity**: Hierarchical role system
- [ ] **Permission Entity**: Granular permissions
- [ ] **Payment Entity**: Transaction data and status
- [ ] **Stock Entity**: Financial instrument data
- [ ] **Session Entity**: Authentication session management

#### 1.3 Define Value Objects
- [ ] **Email**: Validation and formatting
- [ ] **Currency**: Multi-currency support
- [ ] **Price**: Decimal precision handling
- [ ] **DateRange**: Time period handling
- [ ] **UserId**: Typed identifier
- [ ] **PermissionSet**: Permission collections

### Phase 2: Use Cases Implementation (Week 2-3)
#### 2.1 Authentication Use Cases
- [ ] **AuthenticateUser**: Login workflow
- [ ] **RegisterUser**: Registration with role assignment
- [ ] **RefreshSession**: Token refresh logic
- [ ] **ValidatePermission**: Permission checking
- [ ] **LogoutUser**: Session cleanup

#### 2.2 User Management Use Cases (Admin)
- [ ] **CreateUser**: Admin user creation
- [ ] **UpdateUserRole**: Role assignment
- [ ] **UpdatePermissions**: Permission management
- [ ] **DeactivateUser**: User deactivation
- [ ] **AuditUserActivity**: Activity logging

#### 2.3 Payment Use Cases
- [ ] **ProcessPayment**: Payment workflow
- [ ] **VerifyTransaction**: Transaction verification
- [ ] **RefundPayment**: Refund processing
- [ ] **UpdateSubscription**: Subscription management

#### 2.4 Stock/Trading Use Cases
- [ ] **GetStockData**: Real-time data retrieval
- [ ] **SubscribeToUpdates**: WebSocket subscriptions
- [ ] **CalculateAnalytics**: Financial calculations
- [ ] **UpdateRankings**: User ranking updates

### Phase 3: Infrastructure Adapters (Week 3-4)
#### 3.1 Database Adapters
- [ ] **FirestoreUserRepository**: User data persistence
- [ ] **MongoStockRepository**: Stock data persistence  
- [ ] **FirestoreIAMRepository**: Roles and permissions
- [ ] **MongoPaymentRepository**: Payment transaction data

#### 3.2 External Service Adapters
- [ ] **FirebaseAuthService**: Authentication provider
- [ ] **PaymentGatewayService**: Crypto payment processing
- [ ] **StockDataService**: External financial data
- [ ] **EmailService**: Email notifications
- [ ] **WebSocketService**: Real-time data streaming

#### 3.3 Event Handling
- [ ] **DomainEventDispatcher**: Event publishing
- [ ] **EventHandlers**: Async event processing
- [ ] **EventStore**: Event persistence for audit

### Phase 4: Presentation Layer (Week 4-5)
#### 4.1 API Route Adapters
- [ ] **AuthController**: Authentication endpoints
- [ ] **UserController**: User management endpoints
- [ ] **PaymentController**: Payment processing endpoints
- [ ] **StockController**: Stock data endpoints

#### 4.2 React UI Adapters  
- [ ] **UseCaseHooks**: React hooks for use cases
- [ ] **DataPresenters**: Format data for UI consumption
- [ ] **ErrorBoundaries**: Error handling UI
- [ ] **LoadingStates**: Async state management

### Phase 5: Migration & Testing (Week 5-6)
#### 5.1 Gradual Migration
- [ ] **Start with Authentication**: Lowest risk, high impact
- [ ] **Migrate User Management**: Admin features second
- [ ] **Move Payment System**: Critical business logic
- [ ] **Refactor Stock Features**: Real-time complexity last

#### 5.2 Dependency Injection Setup
- [ ] **IoC Container**: Set up dependency injection
- [ ] **Provider Configuration**: Configure DI in apps
- [ ] **Mock Implementations**: For testing
- [ ] **Environment-based Config**: Dev/staging/prod adapters

#### 5.3 Testing Strategy
- [ ] **Unit Tests**: Domain logic and use cases
- [ ] **Integration Tests**: Adapter implementations
- [ ] **Contract Tests**: Port/adapter interfaces
- [ ] **E2E Tests**: Full workflow validation

## Detailed Implementation Examples

### Domain Entity Example
```typescript
// @epsx/domain/entities/User.ts
export class User {
  private constructor(
    private readonly id: UserId,
    private email: Email,
    private profile: UserProfile,
    private roles: Set<Role>,
    private permissions: PermissionSet,
    private subscription: Subscription
  ) {}
  
  static create(userData: CreateUserData): User {
    // Business validation logic
    if (!userData.email.isValid()) {
      throw new InvalidEmailError(userData.email.value);
    }
    
    return new User(
      UserId.generate(),
      userData.email,
      userData.profile,
      new Set([Role.USER]),
      PermissionSet.defaultForRole(Role.USER),
      Subscription.free()
    );
  }
  
  upgradeToRole(newRole: Role): void {
    if (!this.canUpgradeTo(newRole)) {
      throw new RoleUpgradeNotAllowedError(this.getCurrentRole(), newRole);
    }
    
    this.roles.add(newRole);
    this.permissions = PermissionSet.forRoles(this.roles);
    
    // Emit domain event
    DomainEvents.raise(new UserRoleChangedEvent(this.id, newRole));
  }
  
  hasPermission(permission: Permission): boolean {
    return this.permissions.includes(permission);
  }
  
  private canUpgradeTo(role: Role): boolean {
    return RoleHierarchy.canUpgrade(this.getCurrentRole(), role);
  }
  
  private getCurrentRole(): Role {
    return RoleHierarchy.getHighest(this.roles);
  }
}
```

### Use Case Example
```typescript
// @epsx/application/use-cases/AuthenticateUser.ts
export class AuthenticateUserUseCase {
  constructor(
    private userRepository: UserRepositoryPort,
    private authService: AuthServicePort,
    private sessionRepository: SessionRepositoryPort,
    private eventDispatcher: EventDispatcherPort
  ) {}
  
  async execute(request: AuthenticateUserRequest): Promise<AuthenticateUserResponse> {
    // 1. Validate credentials with external auth service
    const authResult = await this.authService.validateCredentials(
      request.email, 
      request.password
    );
    
    if (!authResult.isValid) {
      throw new InvalidCredentialsError();
    }
    
    // 2. Get user from repository
    const user = await this.userRepository.findByEmail(new Email(request.email));
    
    if (!user) {
      throw new UserNotFoundError(request.email);
    }
    
    // 3. Create session
    const session = Session.create(user.id, {
      deviceInfo: request.deviceInfo,
      ipAddress: request.ipAddress,
      expiresAt: DateTime.now().plus({ hours: 24 })
    });
    
    await this.sessionRepository.save(session);
    
    // 4. Emit domain event
    await this.eventDispatcher.dispatch(
      new UserAuthenticatedEvent(user.id, session.id, DateTime.now())
    );
    
    // 5. Return response
    return {
      user: UserDto.fromEntity(user),
      session: SessionDto.fromEntity(session),
      permissions: user.getPermissions().toArray()
    };
  }
}
```

### Repository Port Example
```typescript
// @epsx/application/ports/UserRepositoryPort.ts
export interface UserRepositoryPort {
  findById(id: UserId): Promise<User | null>;
  findByEmail(email: Email): Promise<User | null>;
  save(user: User): Promise<void>;
  delete(id: UserId): Promise<void>;
  findByRole(role: Role): Promise<User[]>;
  findWithPermission(permission: Permission): Promise<User[]>;
}
```

### Infrastructure Adapter Example
```typescript
// @epsx/infrastructure/database/FirestoreUserRepository.ts
export class FirestoreUserRepository implements UserRepositoryPort {
  constructor(private db: Firestore) {}
  
  async findById(id: UserId): Promise<User | null> {
    const doc = await this.db.collection('users').doc(id.value).get();
    
    if (!doc.exists) {
      return null;
    }
    
    return this.mapToEntity(doc.data()!);
  }
  
  async save(user: User): Promise<void> {
    const data = this.mapToDocument(user);
    await this.db.collection('users').doc(user.id.value).set(data);
  }
  
  private mapToEntity(data: any): User {
    return User.reconstruct({
      id: new UserId(data.id),
      email: new Email(data.email),
      profile: new UserProfile(data.profile),
      roles: new Set(data.roles.map((r: string) => new Role(r))),
      permissions: new PermissionSet(data.permissions),
      subscription: new Subscription(data.subscription)
    });
  }
  
  private mapToDocument(user: User): any {
    return {
      id: user.id.value,
      email: user.email.value,
      profile: user.profile.toPlainObject(),
      roles: Array.from(user.roles).map(r => r.value),
      permissions: user.permissions.toArray(),
      subscription: user.subscription.toPlainObject(),
      updatedAt: FieldValue.serverTimestamp()
    };
  }
}
```

### API Controller Example
```typescript
// @epsx/presentation/web-api/AuthController.ts
export class AuthController {
  constructor(
    private authenticateUser: AuthenticateUserUseCase,
    private registerUser: RegisterUserUseCase
  ) {}
  
  async login(req: NextRequest): Promise<NextResponse> {
    try {
      const body = await req.json();
      
      const result = await this.authenticateUser.execute({
        email: body.email,
        password: body.password,
        deviceInfo: req.headers.get('user-agent') || 'Unknown',
        ipAddress: req.ip || 'Unknown'
      });
      
      return NextResponse.json({
        user: result.user,
        session: result.session,
        permissions: result.permissions
      });
      
    } catch (error) {
      return this.handleError(error);
    }
  }
  
  private handleError(error: Error): NextResponse {
    if (error instanceof InvalidCredentialsError) {
      return NextResponse.json(
        { error: 'Invalid credentials' }, 
        { status: 401 }
      );
    }
    
    if (error instanceof UserNotFoundError) {
      return NextResponse.json(
        { error: 'User not found' }, 
        { status: 404 }
      );
    }
    
    // Log unexpected errors
    console.error('Unexpected error:', error);
    
    return NextResponse.json(
      { error: 'Internal server error' }, 
      { status: 500 }
    );
  }
}
```

## Benefits of This Architecture

### Technical Benefits
- **Testability**: Business logic isolated and easily testable
- **Maintainability**: Clear separation of concerns
- **Flexibility**: Easy to swap implementations  
- **Framework Independence**: Core logic not tied to Next.js/React
- **Scalability**: Clean boundaries enable team scaling

### Business Benefits
- **Faster Feature Development**: Clear patterns to follow
- **Lower Bug Risk**: Better separation reduces coupling bugs
- **Easier Onboarding**: Standard patterns for new developers
- **Technology Migration**: Can migrate UI frameworks without changing core logic

## Next Steps

1. **Review and Approval**: Get team sign-off on this plan
2. **Pilot Implementation**: Start with authentication use case
3. **Team Training**: Conduct architecture workshops  
4. **Tool Setup**: Configure testing, DI, and development tools
5. **Begin Phase 1**: Create foundation packages and core entities

---
**Document Status**: Planning Complete - Ready for Implementation  
**Next Review**: After Phase 1 completion