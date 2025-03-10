# Backend Service Architecture

## Service Layer Organization

The backend is organized using a service-layer architecture where business logic is separated from controllers. Each module follows this structure:

### Auth Module
- **AuthService**: Coordinates OAuth and authentication flows
- **TokenService**: Handles token generation and validation
- **SessionService**: Manages user sessions
- **RoleService**: Handles user role management
- **UserManagementService**: Manages user-related operations
- **AuthLoggerService**: Handles authentication logging

### Financial Module
- **FinancialService**: Main service coordinating financial operations
- **AggregationService**: Handles EPS growth calculations and rankings
- **ProcessingService**: Manages processing of financial data
- **PaginationService**: Handles common pagination logic
- **FinancialFetchService**: Manages financial data fetching

### Stock Module
- **StockService**: Main service coordinating stock operations
- **StockDataService**: Handles stock data retrieval and storage
- **StockScrapingService**: Manages stock data scraping operations

### Shared Module
- **DatabaseConfigService**: Manages MongoDB configuration and health checks
- **ErrorHandlingService**: Centralizes error handling and logging
- **HealthController**: Provides application health monitoring endpoints

## Database Configuration

### MongoDB Connection
The application uses MongoDB with the following configuration:
```typescript
MongooseModule.forRootAsync({
  imports: [ConfigModule],
  useClass: DatabaseConfigService
})
```

Required environment variables:
- `MONGODB_URI`: MongoDB connection string

### Health Monitoring
The application includes comprehensive health check endpoints:

- `GET /health`: Overall system health status
- `GET /health/database/details`: Detailed database health information (Admin only)
- `GET /health/database/events`: Database connection events (Admin only)
- `GET /health/readiness`: Application readiness check
- `GET /health/liveness`: Application liveness check

## Error Handling

Services implement consistent error handling:
- Detailed error logging with stack traces
- Custom exceptions with context
- Error transformation for consistent API responses
- Automatic error recovery strategies

## Logging

Each service includes:
- Detailed operation logging
- Error logging with context
- Performance metrics
- Connection event tracking
- Audit trails where necessary

## Configuration

Services can be configured through:
- Environment variables
- Configuration files
- Runtime configuration
- Feature flags

## Testing

Services are designed for testability:
- Unit tests
- Integration tests
- Mocking capabilities
- Test coverage metrics

## Security

- Role-based access control
- Firebase authentication
- Session management
- Admin-only endpoints protection
- Health check access control

## Documentation

Each service includes:
- Method documentation
- Type definitions
- Usage examples
- Error scenarios
- Configuration options

## Monitoring

The application provides:
- Health check endpoints
- Database connection monitoring
- Performance metrics
- Event logging
- Status tracking
