# Breadcrumb Navigation Component

A smart, dynamic breadcrumb navigation component for the EPSX Admin Frontend that automatically generates navigation paths based on current routes.

## Features

### 🧠 Smart Route Detection
- **Automatic breadcrumb generation** from current pathname
- **Dynamic route patterns** for user profile pages (e.g., `/users/[userId]/permissions`)
- **Static route configuration** for all admin pages
- **Fallback parsing** for unknown routes

### 👤 User Data Integration
- **Fetches real user data** for user profile breadcrumbs
- **Shows actual user names** instead of user IDs
- **Loading state indicators** while fetching user data
- **Graceful fallback** if user data fetch fails

### 🎨 Theme-Aware Design
- **Dark mode support** with proper contrast ratios
- **Hover effects** and smooth transitions
- **Loading spinner** integration
- **Consistent styling** with admin layout

## Route Mapping

### Static Routes
```
/                     → Admin > Dashboard
/users               → Admin > User Management
/modules             → Admin > Module Management
/developer-portal    → Admin > Developer Portal
/analytics           → Admin > Analytics Dashboard
/settings            → Admin > General Settings
/billing             → Admin > Billing Management
```

### Dynamic Routes
```
/users/[userId]                → Admin > User Management > [User Name]
/users/[userId]/permissions    → Admin > User Management > [User Name] > Permissions
/users/[userId]/modules        → Admin > User Management > [User Name] > Modules
/users/[userId]/packages       → Admin > User Management > [User Name] > Packages
/users/[userId]/activity       → Admin > User Management > [User Name] > Activity History
```

## Usage

### Basic Implementation
```tsx
import { Breadcrumb } from '@/components/layout/Breadcrumb';

function AdminLayout({ children }) {
  return (
    <div>
      <header>
        <Breadcrumb />
      </header>
      <main>{children}</main>
    </div>
  );
}
```

### Current Integration
The breadcrumb is already integrated into `AdminLayout.tsx` and will automatically show the correct navigation path for all admin pages.

## How It Works

### 1. Route Detection
```typescript
// Extracts user ID from routes like /users/user-123/permissions
const userIdMatch = pathname.match(/^\/users\/([^\/]+)/);
```

### 2. User Data Fetching
```typescript
// Fetches actual user data for display name
useEffect(() => {
  if (currentUserId) {
    getUnifiedUserData(currentUserId).then(result => {
      setUserDisplayName(result.data.display_name || result.data.email);
    });
  }
}, [currentUserId]);
```

### 3. Dynamic Breadcrumb Generation
```typescript
const dynamicRoutePatterns = [
  {
    pattern: /^\/users\/([^\/]+)$/,
    getTitle: (match, displayName) => displayName || 'User Profile',
    getParentPath: () => '/users'
  }
];
```

## Configuration

### Adding New Routes
To add support for new admin routes, update the `routeConfig` object:

```typescript
const routeConfig = {
  // ... existing routes
  '/new-feature': { title: 'New Feature', href: '/new-feature' },
  '/new-feature/sub-page': { title: 'Sub Page', href: '/new-feature/sub-page' },
};
```

### Adding Dynamic Route Patterns
For pages with dynamic segments (like user IDs), add to `dynamicRoutePatterns`:

```typescript
const dynamicRoutePatterns = [
  // ... existing patterns
  {
    pattern: /^\/items\/([^\/]+)$/,
    getTitle: (match) => `Item: ${match[1]}`,
    getParentPath: () => '/items'
  }
];
```

## Examples

### Dashboard Page
```
🏠 Admin > Dashboard
```

### User List Page  
```
🏠 Admin > User Management
```

### User Profile Page
```
🏠 Admin > User Management > John Doe
```

### User Permissions Page
```
🏠 Admin > User Management > John Doe > Permissions
```

## Styling

The breadcrumb uses Tailwind CSS with proper dark mode support:

- **Active page**: `text-blue-600 dark:text-blue-400`
- **Navigation links**: `text-gray-600 dark:text-gray-400` with hover effects
- **Loading spinner**: Animated border with theme-aware colors
- **Icons**: Lucide React icons with consistent sizing

## Performance

- **Lazy loading**: User data is only fetched when on user routes
- **Caching**: Uses the same `getUnifiedUserData` function with built-in caching
- **Debounced updates**: Prevents excessive API calls during route changes
- **Fallback handling**: Graceful degradation if data fetch fails

## Accessibility

- **Semantic navigation**: Uses `<nav>` with `aria-label="Breadcrumb"`
- **Screen reader friendly**: Proper link text and current page indication
- **Keyboard navigation**: Standard tab order and link behavior
- **High contrast**: Meets WCAG contrast requirements in both themes

## Future Enhancements

- **Context awareness**: Could show additional context like permissions
- **Shortcut actions**: Quick actions in breadcrumb items
- **History tracking**: Browser back/forward integration
- **Custom titles**: Allow pages to override breadcrumb titles