# Admin Assignment Scripts

Scripts for managing admin privileges in the EPSX trading platform.

## Files

- `assign-admin.js` - Core Node.js script for assigning admin privileges
- `run-assign-admin.sh` - Bash wrapper script with environment checks
- `README.md` - This documentation

## Quick Usage

```bash
# From project root directory
./scripts/run-assign-admin.sh
```

## Manual Usage

```bash
# Set required environment variable
export FIREBASE_API_KEY="your-firebase-api-key"

# Run the script directly  
node scripts/assign-admin.js
```

## What This Script Does

1. **Finds the user** `jesadakorn.kirtnu@gmail.com` in Firebase
2. **Assigns admin role** via backend API (if available)
3. **Sets custom claims** directly via Firebase Identity Toolkit API
4. **Verifies the assignment** by checking user claims
5. **Provides access summary** for both applications

## Admin Privileges Assigned

### Admin Frontend (@apps/admin-frontend/)
- ✅ User Management (create, edit, delete users)
- ✅ IAM Management (roles, permissions, profiles) 
- ✅ Analytics Dashboard (system metrics, user analytics)
- ✅ Billing Management (subscriptions, payments)
- ✅ Module Management (enable/disable features)
- ✅ Database Administration
- ✅ Developer Portal
- ✅ System Settings

### Frontend (@apps/frontend/)
- ✅ Premium Trading Features
- ✅ Advanced Analytics 
- ✅ EPS Analysis Tools
- ✅ Pattern Recognition
- ✅ All Payment Plans
- ✅ Unrestricted Dashboard Access
- ✅ Admin-level Data Access

## Custom Claims Set

```json
{
  "admin": true,
  "access_level": "full", 
  "role": "admin",
  "permissions": [
    "user_management",
    "iam_management",
    "analytics_access", 
    "billing_management",
    "system_admin",
    "module_management",
    "database_access",
    "developer_portal"
  ]
}
```

## Requirements

- Node.js runtime
- `FIREBASE_API_KEY` environment variable
- Network access to Firebase Identity Toolkit API
- Optional: Backend server running for API-based role assignment

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `FIREBASE_API_KEY` | Yes | Firebase Web API key from console |
| `FIREBASE_PROJECT_ID` | No | Firebase project ID (defaults to 'epsx-trading-platform') |
| `BACKEND_URL` | No | Backend server URL (defaults to 'http://localhost:8080') |

## Getting Firebase API Key

### Method 1: Firebase Console
1. Go to [Firebase Console](https://console.firebase.google.com)
2. Select your EPSX project (or create a new Firebase project)
3. Go to Project Settings (⚙️) > General tab
4. Scroll down to "Your apps" section
5. If no web app exists, click "Add app" and select Web (</>) 
6. Copy the "Web API Key" value from the config
7. Set as `FIREBASE_API_KEY` environment variable

### Method 2: Check Existing Code
The API key might already be in your project files:
- Check `apps/admin-frontend/.env.local`
- Check `apps/frontend/.env.local` 
- Check `apps/backend/.env`
- Look for `FIREBASE_API_KEY` or `NEXT_PUBLIC_FIREBASE_API_KEY`

### Method 3: Enable Firebase Authentication
If you have a valid API key but getting "API key not valid" errors:
1. Go to Firebase Console > Authentication
2. Click "Get started" if not already enabled
3. Go to Settings tab > Authorized domains
4. Add your domains (localhost for development)
5. Make sure Identity Toolkit API is enabled in Google Cloud Console

## Troubleshooting

### "User not found" Error
- Verify the email address exists in Firebase Authentication
- Check that `FIREBASE_API_KEY` is correct
- Ensure the user has been created in Firebase

### "FIREBASE_API_KEY not set" Error  
- Set the environment variable: `export FIREBASE_API_KEY="your-key"`
- Or add it to your `.env` file: `FIREBASE_API_KEY=your-key`

### "Assignment completed but verification failed"
- The assignment likely worked, but Firebase may need time to propagate
- User should log out and log back in
- Custom claims take effect on next authentication

### Network/API Errors
- Check internet connectivity
- Verify Firebase project is active
- Ensure API key has proper permissions

## Security Notes

- This script assigns full admin privileges - use responsibly
- Keep the Firebase API key secure and never commit to version control
- Consider implementing audit logging for admin privilege changes
- Review admin access periodically

## Testing After Assignment

1. User logs out of both applications
2. User logs back in
3. Test access to admin-frontend features:
   - Navigate to `/users` (User Management)
   - Navigate to `/iam` (IAM Management) 
   - Navigate to `/analytics` (Analytics Dashboard)
4. Test access to frontend premium features:
   - Advanced analytics tools
   - EPS analysis features
   - Pattern recognition tools