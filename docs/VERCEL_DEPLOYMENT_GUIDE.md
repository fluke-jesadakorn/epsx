# Vercel Deployment Guide for EPSX

## Prerequisites

1. **Vercel CLI** (optional but recommended)
   ```bash
   npm i -g vercel
   ```

2. **Environment Variables** - Set up the following in Vercel dashboard:

### Required Environment Variables

```bash
# App Configuration
NODE_ENV=production
NEXT_PUBLIC_SITE_URL=https://your-domain.vercel.app
NEXT_PUBLIC_API_URL=https://your-backend-api.com

# Google OAuth
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret

# MusePay Configuration
MUSEPAY_PARTNER_ID=your-musepay-partner-id
MUSEPAY_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----your-musepay-private-key-----END PRIVATE KEY-----"
MUSEPAY_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----your-musepay-public-key-----END PUBLIC KEY-----"
MUSEPAY_API_URL=https://api.test.topay.mobi/v1
NEXT_PUBLIC_MUSEPAY_NOTIFY_URL=https://your-domain.vercel.app/api/v1/webhook/musepay

# Database
DATABASE_URL="postgresql://user:password@host:5432/epsx"

# Email Service
EMAIL_SERVICE_API_KEY="your-email-service-api-key"
```

## Deployment Steps

### Option 1: GitHub Integration (Recommended)

1. **Connect Repository to Vercel**
   - Go to [Vercel Dashboard](https://vercel.com/dashboard)
   - Click "New Project"
   - Import your GitHub repository
   - Select the root directory `/Users/fluke/Desktop/Work/Outsource/epsx`

2. **Configure Build Settings**
   - **Framework Preset**: Next.js
   - **Root Directory**: `apps/frontend`
   - **Build Command**: `cd ../.. && pnpm build:frontend`
   - **Output Directory**: `.next`
   - **Install Command**: `cd ../.. && pnpm install`

3. **Set Environment Variables**
   - Add all required environment variables in the Vercel dashboard
   - Go to Project Settings → Environment Variables

4. **Deploy**
   - Click "Deploy"
   - Vercel will automatically deploy on every push to the main branch

### Option 2: Vercel CLI

1. **Login to Vercel**
   ```bash
   vercel login
   ```

2. **Navigate to Project**
   ```bash
   cd /Users/fluke/Desktop/Work/Outsource/epsx/apps/frontend
   ```

3. **Initialize Project**
   ```bash
   vercel
   ```
   - Follow the prompts
   - Set root directory to project root
   - Configure build settings as above

4. **Set Environment Variables**
   ```bash
   vercel env add NEXT_PUBLIC_SITE_URL
   vercel env add NEXT_PUBLIC_API_URL
   vercel env add GOOGLE_CLIENT_ID
   # ... add all other environment variables
   ```

5. **Deploy**
   ```bash
   vercel --prod
   ```

## Configuration Files

The following configuration files have been created for Vercel deployment:

### `/vercel.json` (Root)
```json
{
  "version": 2,
  "builds": [
    {
      "src": "apps/frontend/package.json",
      "use": "@vercel/next",
      "config": {
        "distDir": ".next"
      }
    }
  ],
  "routes": [
    {
      "src": "/(.*)",
      "dest": "apps/frontend/$1"
    }
  ],
  "env": {
    "NODE_ENV": "production"
  },
  "functions": {
    "apps/frontend/app/api/**/*.js": {
      "maxDuration": 30
    }
  },
  "crons": [],
  "regions": ["iad1"],
  "framework": "nextjs"
}
```

### `/apps/frontend/vercel.json`
```json
{
  "buildCommand": "cd ../.. && pnpm build:frontend",
  "devCommand": "cd ../.. && pnpm dev:frontend",
  "installCommand": "cd ../.. && pnpm install",
  "outputDirectory": ".next",
  "framework": "nextjs",
  "functions": {
    "app/api/**/*.js": {
      "maxDuration": 30
    }
  },
  "env": {
    "NEXT_PUBLIC_SITE_URL": "@next_public_site_url",
    "NEXT_PUBLIC_API_URL": "@next_public_api_url"
  },
  "build": {
    "env": {
      "NODE_ENV": "production"
    }
  }
}
```

## Build Optimizations Applied

1. **Removed `output: 'standalone'`** from Next.js config for Vercel compatibility
2. **Fixed SSR issues** with `window.location` references
3. **Resolved cookie usage** in static generation
4. **Bundle optimization** with code splitting
5. **Performance monitoring** configured for production

## Post-Deployment

### 1. Domain Configuration
- Add custom domain in Vercel dashboard if needed
- Update `NEXT_PUBLIC_SITE_URL` to match your domain

### 2. Database Setup
- Ensure your PostgreSQL database is accessible from Vercel
- Consider using Vercel Postgres or external providers like Supabase/PlanetScale

### 3. Backend API
- Deploy your Rust backend separately (Railway, Fly.io, AWS, etc.)
- Update `NEXT_PUBLIC_API_URL` to point to your backend

### 4. Monitoring
- Enable Vercel Analytics
- Configure performance monitoring
- Set up error tracking (Sentry, etc.)

## Troubleshooting

### Build Errors

1. **Dynamic Server Usage Errors**
   - Some routes still use cookies for authentication
   - These routes will be server-rendered (marked with ƒ)
   - This is expected behavior for authenticated routes

2. **Environment Variable Issues**
   - Ensure all required variables are set in Vercel dashboard
   - Use `NEXT_PUBLIC_` prefix for client-side variables

3. **Build Command Issues**
   - Ensure pnpm workspace configuration is correct
   - Check that all dependencies are properly installed

### Performance Issues
- Monitor First Load JS size (currently ~261 kB)
- Consider implementing more code splitting if needed
- Enable compression and caching headers

## Security Considerations

1. **Environment Variables**
   - Never commit sensitive keys to repository
   - Use Vercel's encrypted environment variables

2. **API Security**
   - Ensure your backend has proper CORS configuration
   - Implement rate limiting on API endpoints

3. **Authentication**
   - Verify OAuth redirect URLs match your domain
   - Update webhook URLs for payment processing

## Success Metrics

✅ **Build Status**: Successful  
✅ **Static Generation**: 34/43 pages statically generated  
✅ **Bundle Size**: ~261 kB first load JS  
✅ **Performance**: Optimized code splitting implemented  

## Next Steps

1. Test the deployment with staging environment variables
2. Verify all authentication flows work correctly
3. Test payment webhook integrations
4. Monitor performance and error rates
5. Set up CI/CD pipeline for automated deployments