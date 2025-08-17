# EPSX Frontend

A standalone Next.js 15 application for the EPSX trading platform, providing advanced stock analysis, earnings per share (EPS) tracking, and premium trading features for retail and professional traders.

## Features

- **Real-time Stock Rankings**: Live stock performance rankings with advanced filtering
- **EPS Analysis**: Comprehensive earnings per share tracking and analytics
- **Pattern Recognition**: AI-powered technical analysis and pattern detection
- **Premium Analytics**: Advanced charting and financial metrics for subscribers
- **Payment Integration**: Seamless subscription and payment processing
- **User Dashboard**: Personalized trading dashboard and portfolio management
- **Mobile Responsive**: Optimized for all devices and screen sizes

## Tech Stack

- **Framework**: Next.js 15.4.6 with App Router
- **Runtime**: React 19.1.0 with Server Components
- **Styling**: Tailwind CSS 4.0.15 with custom components
- **Authentication**: Firebase Auth + JWT tokens
- **State Management**: Zustand + SWR for server state
- **Charts**: Recharts for financial visualizations
- **Forms**: React Hook Form + Zod validation
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
   git clone https://github.com/fluke-jesadakorn/epsx-frontend
   cd epsx-frontend
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
   - Local: http://localhost:3000
   - Registration/Login available for new users

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
NEXTAUTH_URL=http://localhost:3000
NEXTAUTH_SECRET=your_nextauth_secret

# Payment Configuration
STRIPE_PUBLIC_KEY=your_stripe_public_key
STRIPE_SECRET_KEY=your_stripe_secret_key

# Analytics
NEXT_PUBLIC_GA_ID=your_google_analytics_id
```

## Available Scripts

### Development
```bash
npm run dev          # Start development server on port 3000
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
│   ├── dashboard/         # User dashboard
│   ├── analytics/         # Analytics and charts
│   ├── payment/           # Payment and subscription
│   └── settings/          # User settings
├── components/            # React components
│   ├── home/             # Landing page components
│   ├── auth/             # Authentication components
│   ├── analytics/        # Analytics and charting
│   ├── payment/          # Payment components
│   ├── ui/               # Reusable UI components
│   └── guards/           # Access control guards
├── lib/                   # Utility libraries
│   ├── auth/             # Authentication utilities
│   ├── api/              # API client layer
│   └── utils/            # Helper functions
├── hooks/                 # Custom React hooks
├── services/             # API service layer
├── config/               # Configuration files
└── __test__/             # Test files
```

## User Features

### Free Tier
- Basic stock rankings
- Limited EPS data
- Standard analytics
- Mobile access

### Premium Tiers

**Bronze Tier**
- Extended stock rankings
- Basic EPS analysis
- Historical data (6 months)
- Email alerts

**Silver Tier**
- Advanced analytics
- EPS growth analysis
- Historical data (2 years)
- Custom watchlists

**Gold Tier**
- Pattern recognition
- Predictive analytics
- Historical data (5 years)
- API access

**Platinum Tier**
- Real-time data
- Advanced AI insights
- Unlimited historical data
- Priority support

## API Integration

The application communicates with the backend through:

- **REST APIs**: Stock data and user management
- **WebSocket**: Real-time price updates
- **Server Actions**: Form submissions and mutations
- **OAuth 2.0**: Secure authentication flow

## Authentication Flow

1. **User Registration**: Email/password or Google OAuth
2. **Email Verification**: Required for account activation
3. **JWT Tokens**: Backend issues access tokens
4. **Session Management**: Client-side session storage
5. **Subscription Validation**: Package tier verification

## Payment Integration

- **Stripe**: Credit card processing
- **Subscription Management**: Automatic billing cycles
- **Package Upgrades**: Seamless tier upgrades
- **Payment History**: Transaction tracking
- **Refund Processing**: Automated refund handling

## Deployment

### Docker

```bash
# Build image
docker build -t epsx-frontend .

# Run container
docker run -p 3000:3000 \
  -e NEXT_PUBLIC_BACKEND_URL=https://api.epsx.io \
  epsx-frontend
```

### Google Cloud Run

```bash
# Deploy to Cloud Run
gcloud run deploy epsx-frontend \
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
- **XSS Protection**: Input sanitization and validation
- **JWT Validation**: All authenticated requests verified
- **Rate Limiting**: API request throttling
- **Data Encryption**: Sensitive data encrypted at rest

## Performance

- **Server Components**: Optimized rendering strategy
- **Code Splitting**: Automatic route-based splitting
- **Image Optimization**: Next.js automatic optimization
- **Bundle Analysis**: Regular bundle size monitoring
- **Caching**: Multi-tier caching strategy
- **CDN**: Global content delivery network

## Analytics & Monitoring

- **Google Analytics**: User behavior tracking
- **Error Monitoring**: Real-time error reporting
- **Performance Metrics**: Core Web Vitals monitoring
- **User Analytics**: Subscription and usage metrics

## Mobile Experience

- **Responsive Design**: Mobile-first approach
- **Touch Gestures**: Optimized for mobile interactions
- **Offline Support**: Service worker for offline functionality
- **PWA Ready**: Progressive Web App capabilities

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
- Optimize for mobile performance
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
- Check backend connectivity
- Confirm JWT token validity

**Payment Integration Issues**
- Verify Stripe configuration
- Check webhook endpoints
- Confirm environment variables

**Performance Issues**
- Run bundle analysis
- Check for memory leaks
- Optimize component re-renders

## Support

- **Documentation**: See `/docs` directory
- **Issues**: GitHub Issues
- **Support**: support@epsx.io
- **Security**: security@epsx.io

## License

This project is licensed under the UNLICENSED License - see the LICENSE file for details.

---

**EPSX Team** - Empowering traders with advanced analytics