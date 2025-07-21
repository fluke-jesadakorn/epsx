# EPSX Shared Package - Database Seeding

This package provides comprehensive database seeding functionality for the EPSX platform. It includes both static configuration data (stored as JSON files) and dynamic data seeding for Firebase Firestore.

## Features

- **Comprehensive Seeding**: Seeds all essential collections for a complete EPSX setup
- **Modular Design**: Seed individual collections or all at once
- **Environment Support**: Different configurations for development, test, and production
- **Data Validation**: Validates seed data before execution
- **CLI Tool**: Easy-to-use command-line interface
- **Type Safety**: Full TypeScript support with comprehensive type definitions

## Architecture

### Static Data (JSON Files)
- **Roles** (`data/roles.json`) - User roles and their basic permissions
- **Permissions** (`data/permissions.json`) - Granular permission definitions
- **Package Permissions** (`data/package-permissions.json`) - Feature access by subscription tier

### Dynamic Data (Firebase Collections)
- **IAM System** - Users, sessions, audit logs
- **Organizations** - Company settings, invitations, user preferences
- **Content Management** - Articles, media, categories
- **Analytics** - Usage tracking, system metrics, feature analytics
- **Notifications** - Email templates, notifications, message queue
- **Configuration** - System settings, feature flags, integrations

## Installation

```bash
# Install dependencies
pnpm install

# Build the package
pnpm build
```

## Usage

### CLI Commands

#### Initialize Complete Project
```bash
# Seed all collections for development
pnpm seed:init

# Seed for production environment
pnpm seed:init --environment production --force

# Quiet mode
pnpm seed:init --no-verbose
```

#### Seed Specific Collections
```bash
# Seed only IAM data
tsx src/seeding/cli.ts seed iam

# Seed multiple collections
tsx src/seeding/cli.ts seed iam organizations content

# With options
tsx src/seeding/cli.ts seed analytics --environment test --force
```

#### Validate Data
```bash
# Validate all seed data files
pnpm seed:validate
```

#### List Collections
```bash
# Show all available collections
pnpm seed:list
```

#### Clear Data (DANGEROUS)
```bash
# Clear all seeded data
tsx src/seeding/cli.ts clear --confirm
```

### Programmatic Usage

```typescript
import { getFirestore } from 'firebase/firestore';
import { seedInitialProject, createSeedManager } from '@epsx/shared';

// Initialize Firebase
const db = getFirestore();

// Seed entire project
const results = await seedInitialProject(db, {
  environment: 'development',
  force: false,
  verbose: true
});

// Custom seeding
const manager = createSeedManager(db, { environment: 'development' });
await manager.runAll();

// Seed specific collections
const specificResults = await seedCollections(db, ['iam', 'content'], {
  environment: 'production',
  force: true
});
```

## Environment Configuration

Set these environment variables for Firebase connection:

```bash
FIREBASE_API_KEY=your_api_key
FIREBASE_AUTH_DOMAIN=your_project.firebaseapp.com
FIREBASE_PROJECT_ID=your_project_id
FIREBASE_STORAGE_BUCKET=your_project.appspot.com
FIREBASE_MESSAGING_SENDER_ID=123456789
FIREBASE_APP_ID=1:123456789:web:abcdef123456
```

## Seeded Data Overview

### IAM System (9 documents)
- **3 Roles**: admin, manager, editor, viewer, beta-tester, support, api-user
- **24 Permissions**: Granular access controls across all features
- **3 Users**: Complete user profiles with different access levels
- **2 Sessions**: Active user sessions with device info
- **3 Audit Logs**: System initialization and login events

### Organizations (7 documents)
- **2 Organizations**: Main org and demo org with different settings
- **2 Invitations**: Pending team invitations
- **3 User Preferences**: UI, privacy, and feature preferences

### Content Management (15 documents)
- **5 Categories**: Organized content hierarchy
- **4 Media Files**: Logos, images, videos, diagrams
- **6 Content Items**: Documentation, tutorials, announcements

### Analytics (34 documents)
- **21 Usage Analytics**: 7 days of usage data for 3 users
- **7 System Metrics**: Daily system performance metrics
- **3 Feature Usage**: Real-time feature interaction tracking
- **3 Usage Tracking**: Subscription usage and limits

### Notifications (18 documents)
- **6 Email Templates**: Welcome, invitation, password reset, etc.
- **8 Notifications**: Various notification types for users
- **4 Message Queue**: Email and push notification queue

### Configuration (19 documents)
- **6 System Settings**: General, security, email, storage, analytics, billing
- **8 Feature Flags**: Beta features, new UI, collaboration tools
- **5 Integrations**: Stripe, SendGrid, AWS S3, Google Analytics, Slack

## Data Structure

### User Roles Hierarchy
```
Admin (Full Access)
├── Manager (Team Management)
├── Editor (Content Management)
├── Support (Customer Support)
├── Beta Tester (Feature Testing)
├── API User (Programmatic Access)
└── Viewer (Read-only)
```

### Package Levels
```
Enterprise (Unlimited)
├── Gold (Advanced Features)
├── Silver (Standard Features)
├── Bronze (Basic Features)
└── Free (Limited Features)
```

## Customization

### Adding New Roles
Edit `src/seeding/data/roles.json`:

```json
{
  "id": "custom-role",
  "name": "Custom Role",
  "description": "Custom role description",
  "permissions": ["permission:id"],
  "isSystem": false
}
```

### Adding New Permissions
Edit `src/seeding/data/permissions.json`:

```json
{
  "id": "custom:permission",
  "featureId": "custom_feature",
  "permission": "access",
  "description": "Custom permission description",
  "category": "custom"
}
```

### Extending Package Permissions
Edit `src/seeding/data/package-permissions.json`:

```json
{
  "SILVER": [
    {
      "featureId": "new_feature",
      "permission": "access",
      "limits": { "usage": 100 }
    }
  ]
}
```

## Development

### File Structure
```
src/seeding/
├── data/                     # Static JSON configuration
│   ├── roles.json
│   ├── permissions.json
│   └── package-permissions.json
├── types/                    # TypeScript definitions
│   └── index.ts
├── seeders/                  # Individual seeder classes
│   ├── base.ts              # Base seeder class
│   ├── iam.ts               # IAM system seeder
│   ├── organization.ts      # Organization seeder
│   ├── content.ts           # Content seeder
│   ├── analytics.ts         # Analytics seeder
│   ├── notifications.ts     # Notification seeder
│   ├── configuration.ts     # Configuration seeder
│   └── index.ts             # Seeder exports
├── index.ts                 # Main seeding functions
└── cli.ts                   # Command-line interface
```

### Adding New Seeders

1. Create a new seeder class extending `BaseSeeder`
2. Implement the `seed()` method and `collectionName` getter
3. Add the seeder to `createSeedManager()` in `index.ts`
4. Update the CLI help text if needed

```typescript
export class CustomSeeder extends BaseSeeder {
  get collectionName(): string {
    return 'custom';
  }

  async seed(): Promise<SeedResult> {
    // Implementation
  }
}
```

## Best Practices

1. **Environment Separation**: Use different configurations for dev/test/prod
2. **Force Mode**: Only use `--force` when you want to overwrite existing data
3. **Validation**: Always run `validate` before seeding production
4. **Backup**: Back up production data before running seeders
5. **Monitoring**: Check seeding results and handle failures appropriately

## Troubleshooting

### Common Issues

1. **Firebase Connection**: Ensure environment variables are set correctly
2. **Permission Errors**: Check Firebase security rules and authentication
3. **Data Validation**: Run validation command to check JSON files
4. **Memory Issues**: For large datasets, consider seeding in batches

### Error Messages

- `Failed to load static seed data files`: Check JSON file syntax
- `Firebase initialization failed`: Verify environment variables
- `Collection already exists`: Use `--force` to overwrite
- `Invalid role/permission structure`: Check data format in JSON files

## License

This package is part of the EPSX platform and follows the same licensing terms.
