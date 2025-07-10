# EPSX Monorepo

A modern monorepo for stock analytics and management, featuring a user-facing frontend, an admin dashboard, and a secure Rust backend. Built with Next.js, React, Tailwind CSS, Firebase, and Rust, this project is designed for scalability, security, and developer productivity.

---

## Monorepo Structure

```
.
├── apps/
│   ├── frontend/         # User-facing web app (Next.js, React, Tailwind CSS, Firebase)
│   ├── admin-frontend/   # Admin dashboard (Next.js, React, Tailwind CSS)
│   └── backend/          # Rust backend for authentication, session management, and API
├── packages/             # Shared libraries (UI, config, types, utils, etc.)
├── package.json          # Monorepo scripts and configuration
├── pnpm-workspace.yaml   # pnpm workspace configuration
└── turbo.json            # TurboRepo configuration
```

---

## Features

### Frontend (`apps/frontend`)
- Modern Next.js app with React and Tailwind CSS
- User authentication via Google OAuth (Firebase)
- Stock analytics and research tools
- Responsive UI and dark mode support

### Admin Frontend (`apps/admin-frontend`)
- Admin dashboard for managing users and data
- Built with Next.js, React, and Tailwind CSS
- Uses shared UI and config packages

### Backend (`apps/backend`)
- Rust-based API server
- Secure authentication with Firebase Admin SDK
- Session management with HTTP-only cookies
- Google OAuth token verification
- RESTful API endpoints for authentication and session management

---

## Technologies Used

- **Frontend:** Next.js, React, Tailwind CSS, Firebase, SWR, Zustand, Radix UI
- **Backend:** Rust, Firebase Admin SDK, Google OAuth
- **Monorepo:** pnpm, TurboRepo, TypeScript, Prettier, ESLint

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) (install via `npm i -g pnpm`)
- [Rust](https://www.rust-lang.org/tools/install) (for backend)

### Installation

1. **Clone the repository**
   ```bash
   git clone git@github.com:fluke-jesadakorn/epsx.git
   cd epsx
   ```

2. **Install dependencies**
   ```bash
   pnpm install
   ```

3. **Setup environment variables**
   - Copy `.env.example` to `.env` in each app as needed
   - Fill in required values (see `.env.example` for details)

4. **Build and run apps**

   - **Frontend**
     ```bash
     pnpm dev:fe
     # or
     cd apps/frontend
     pnpm dev
     ```

   - **Admin Frontend**
     ```bash
     pnpm dev:admin
     # or
     cd apps/admin-frontend
     pnpm dev
     ```

   - **Backend**
     ```bash
     cd apps/backend
     cp .env.example .env
     cargo build
     cargo run
     ```

---

## Backend Authentication Flow

- Uses Firebase Admin SDK for token verification
- Google OAuth for user login
- Secure session management with HTTP-only cookies
- RESTful API endpoints for authentication and session validation

**Example Endpoints:**
- `POST /v1/auth/google/init` — Initiate Google OAuth flow
- `GET /v1/auth/google/callback` — Handle OAuth callback and set session
- `GET /v1/auth/session/validate` — Validate active session
- `POST /v1/auth/logout` — Logout and clear session

---

## Environment Variables

Each app has its own `.env.example` file. Common variables include:

- Firebase project credentials
- Google OAuth credentials
- Session and cookie configuration

---

## Scripts

- `pnpm dev` — Start all apps in development mode
- `pnpm build` — Build all apps and packages
- `pnpm lint` — Lint all code
- `pnpm format` — Format codebase with Prettier
- `pnpm type-check` — Type check all packages

See `package.json` for more scripts and details.

---

## Contributing

- Use `pnpm` for dependency management.
- Run `pnpm lint` and `pnpm format` before committing.
- Follow code style enforced by Prettier and ESLint.
- Pull requests are welcome!

---

## License

[MIT](LICENSE) (or specify your license here)

---

## Acknowledgments

- Built with [Next.js](https://nextjs.org/), [React](https://react.dev/), [Tailwind CSS](https://tailwindcss.com/), [Firebase](https://firebase.google.com/), [Rust](https://www.rust-lang.org/), and [TurboRepo](https://turbo.build/).
