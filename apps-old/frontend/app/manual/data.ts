export interface Feature {
  id: string;
  name: string;
  desc: string;
  route: string;
  screenshots: string[];
  category: string;
}

export const CATEGORIES = [
  'Public',
  'Auth',
  'Dashboard',
  'Analytics',
  'Plans',
  'Portfolio',
  'Notifications',
  'Developer',
] as const;

export const FEATURES: Feature[] = [
  // Public
  { id: 'home', name: 'Home', desc: 'The landing page displays the hero section with platform tagline, primary navigation bar, and an overview of key features. Visitors see call-to-action buttons for signing up and exploring analytics.', route: '/', screenshots: ['home'], category: 'Public' },
  { id: 'about', name: 'About', desc: 'The about page presents platform background, mission statement, and team information. Sections describe the technology stack, partnerships, and the roadmap for upcoming features.', route: '/about', screenshots: ['about'], category: 'Public' },
  { id: 'terms', name: 'Terms of Service', desc: 'The legal terms page shows the full terms and conditions governing platform use. Users can read through sections covering account responsibilities, data usage, and dispute resolution.', route: '/terms', screenshots: ['terms'], category: 'Public' },
  { id: 'privacy', name: 'Privacy Policy', desc: 'The privacy policy page outlines how user data is collected, stored, and protected. Sections cover cookie usage, third-party integrations, and data retention periods.', route: '/privacy', screenshots: ['privacy'], category: 'Public' },
  { id: 'offline', name: 'Offline', desc: 'The PWA offline fallback page is shown when the user loses internet connectivity. It displays a friendly message indicating the app is offline and will reconnect automatically.', route: '/offline', screenshots: ['offline'], category: 'Public' },
  { id: 'access-denied', name: 'Access Denied', desc: 'This error page appears when a user attempts to access a route they lack permissions for. It shows the denied resource and suggests contacting an admin or upgrading their plan.', route: '/access-denied', screenshots: ['access-denied'], category: 'Public' },

  // Auth
  { id: 'auth', name: 'Authentication', desc: 'The Web3 authentication page presents wallet connection options via RainbowKit. Users can connect MetaMask, WalletConnect, or other providers, then sign a SIWE message to authenticate.', route: '/auth', screenshots: ['auth'], category: 'Auth' },

  // Dashboard
  { id: 'dashboard', name: 'Dashboard', desc: 'The main user dashboard displays portfolio summary stats, a watchlist of tracked stocks, and recent activity feed. Key metrics like total portfolio value and daily change are shown at the top.', route: '/dashboard', screenshots: ['dashboard'], category: 'Dashboard' },
  { id: 'account', name: 'Account Overview', desc: 'The account overview tab shows the user\'s current subscription plan, wallet address, and access level. Summary cards display plan expiration date, feature entitlements, and quick links to manage settings.', route: '/account', screenshots: ['account'], category: 'Dashboard' },
  { id: 'account-payments', name: 'Payment History', desc: 'User navigates to the Payments tab on the account page. The tab displays a chronological list of past transactions including amounts, dates, transaction hashes, and payment status badges.', route: '/account', screenshots: ['account-payments'], category: 'Dashboard' },
  { id: 'account-prefs', name: 'Notification Preferences', desc: 'User opens the Preferences tab and toggles notification settings. The UI shows switches for email alerts, push notifications, and in-app notifications, with the toggled switch reflecting the updated state.', route: '/account', screenshots: ['account-prefs'], category: 'Dashboard' },
  { id: 'account-credits', name: 'Credits', desc: 'The credits page displays the user\'s current credit balance, recent usage history, and purchase options. A usage chart shows credit consumption over time alongside available top-up packages.', route: '/account/credits', screenshots: ['account-credits'], category: 'Dashboard' },
  { id: 'profile', name: 'Profile', desc: 'The profile page shows the user\'s display name, connected wallet address, and account metadata. Read-only fields display registration date, last login, and current plan tier.', route: '/profile', screenshots: ['profile'], category: 'Dashboard' },
  { id: 'profile-edit', name: 'Edit Profile', desc: 'User clicks the Edit button on the profile page and types a new display name into the name input field. The form shows the editable field with the updated text and Save/Cancel action buttons.', route: '/profile', screenshots: ['profile-edit'], category: 'Dashboard' },

  // Analytics
  { id: 'analytics-default', name: 'Stock Rankings', desc: 'The default analytics view displays a paginated table of ranked stocks with columns for ticker, company name, price, daily change, volume, and composite score. Data loads with the default sort order.', route: '/analytics', screenshots: ['analytics-default'], category: 'Analytics' },
  { id: 'analytics-search', name: 'Search Stocks', desc: 'User types "AAPL" into the search input above the rankings table. The table filters in real-time to show only rows matching the query, displaying Apple Inc. and related tickers. The search input shows the active query text.', route: '/analytics', screenshots: ['analytics-search'], category: 'Analytics' },
  { id: 'analytics-filter-country', name: 'Filter by Country', desc: 'User clicks the Country filter button to open the country selection dropdown. The filter UI displays available country options, allowing the user to narrow rankings to stocks from a specific market.', route: '/analytics', screenshots: ['analytics-filter-country'], category: 'Analytics' },
  { id: 'analytics-filter-sector', name: 'Filter by Sector', desc: 'User clicks the Sector filter button to open the sector selection dropdown. Available sectors like Technology, Healthcare, and Finance are displayed, letting the user view rankings for a specific industry.', route: '/analytics', screenshots: ['analytics-filter-sector'], category: 'Analytics' },
  { id: 'analytics-sort', name: 'Sort Column', desc: 'User clicks a column header (e.g., Price or Change) to sort the rankings table. The column shows a sort direction indicator and the table rows reorder according to the selected column values.', route: '/analytics', screenshots: ['analytics-sort'], category: 'Analytics' },
  { id: 'analytics-pagination', name: 'Pagination', desc: 'User clicks the Next page button or page number in the pagination controls below the table. The table loads the next set of results and the pagination indicator updates to reflect the current page.', route: '/analytics', screenshots: ['analytics-pagination'], category: 'Analytics' },

  // Plans
  { id: 'plans', name: 'Plans', desc: 'The plans page presents available subscription tiers as side-by-side cards with pricing, feature lists, and comparison highlights. Each card shows the plan name, monthly price, and a Subscribe button.', route: '/plans', screenshots: ['plans'], category: 'Plans' },
  { id: 'payment', name: 'Payment', desc: 'The crypto payment checkout page shows the selected plan summary, total amount in USD and equivalent crypto, and wallet connection status. Users review the order before confirming the blockchain transaction.', route: '/payment', screenshots: ['payment'], category: 'Plans' },
  { id: 'payment-detail', name: 'Payment Detail', desc: 'The payment processing page for a specific plan and payment type displays transaction details, confirmation steps, and real-time status updates as the blockchain transaction is submitted and confirmed.', route: '/payment/[type]/[id]', screenshots: ['payment-detail'], category: 'Plans' },

  // Portfolio
  { id: 'portfolio', name: 'Portfolio', desc: 'The portfolio page displays the user\'s stock holdings in a table with columns for ticker, shares held, average cost, current value, and profit/loss. Summary cards at the top show total portfolio value and overall performance.', route: '/portfolio', screenshots: ['portfolio'], category: 'Portfolio' },
  { id: 'portfolio-search', name: 'Search Portfolio', desc: 'User types "AAPL" into the portfolio search input to filter their holdings. The table narrows to display only matching positions, showing the search query in the input and the filtered result count.', route: '/portfolio', screenshots: ['portfolio-search'], category: 'Portfolio' },
  { id: 'permissions', name: 'Permissions', desc: 'The permissions page lists the user\'s current feature entitlements granted by their subscription plan. Each permission shows the resource name, access level, and expiration date if applicable.', route: '/permissions', screenshots: ['permissions'], category: 'Portfolio' },

  // Notifications
  { id: 'notifications-default', name: 'Notifications', desc: 'The notification center displays all notifications in a chronological list with type icons, priority badges, timestamps, and read/unread indicators. Filter controls for type and priority appear above the list.', route: '/notifications', screenshots: ['notifications-default'], category: 'Notifications' },
  { id: 'notifications-filter-type', name: 'Filter by Type', desc: 'User clicks the Type filter and selects "Security" to narrow the notification list. Only security-related notifications are displayed, and the active filter chip shows the selected type.', route: '/notifications', screenshots: ['notifications-filter-type'], category: 'Notifications' },
  { id: 'notifications-filter-priority', name: 'Filter by Priority', desc: 'User clicks the Priority filter and selects "High" to show only urgent notifications. The list updates to display high-priority items, each marked with a colored priority badge.', route: '/notifications', screenshots: ['notifications-filter-priority'], category: 'Notifications' },
  { id: 'notifications-search', name: 'Search Notifications', desc: 'User types "security" into the notification search input. The list filters to show only notifications whose title or body contains the search term, with the query visible in the input field.', route: '/notifications', screenshots: ['notifications-search'], category: 'Notifications' },
  { id: 'notifications-empty', name: 'Empty State', desc: 'The notification center with no notifications displays an empty state illustration and message. This view appears when all notifications have been cleared or when using filters that match no results.', route: '/notifications', screenshots: ['notifications-empty'], category: 'Notifications' },

  // Developer
  { id: 'developer', name: 'Developer Portal', desc: 'The developer portal overview shows active API keys with their usage stats, rate limit status, and creation dates. Summary cards display total API calls, remaining quota, and quick links to documentation.', route: '/developer', screenshots: ['developer'], category: 'Developer' },
  { id: 'developer-create-key', name: 'Create API Key', desc: 'User clicks the Create button on the developer portal to open the API key creation dialog. The modal displays fields for key name, permission scopes, and expiration settings before generating a new key.', route: '/developer', screenshots: ['developer-create-key'], category: 'Developer' },
  { id: 'developer-docs', name: 'API Documentation', desc: 'The interactive API documentation page presents available endpoints grouped by module. Each endpoint card shows the HTTP method, path, description, request parameters, and expandable code samples.', route: '/developer/docs', screenshots: ['developer-docs'], category: 'Developer' },
  { id: 'developer-usage', name: 'API Usage', desc: 'The API usage page displays call volume charts over time, current rate limit consumption, and per-endpoint breakdown tables. Usage metrics include response times, error rates, and quota utilization.', route: '/developer/usage', screenshots: ['developer-usage'], category: 'Developer' },
];
