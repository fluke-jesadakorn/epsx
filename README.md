
# EPSX Frontend

A modern stock analytics and research web app built with Next.js, React, Tailwind CSS, and Firebase.

---

## Features

- Next.js + React for fast, scalable UI
- Tailwind CSS for responsive design and dark mode
- User authentication via Google OAuth (Firebase)
- Stock analytics and research tools
- State management with SWR and Zustand
- Radix UI for accessible components

---

## Technologies

- **Framework:** Next.js, React
- **Styling:** Tailwind CSS
- **Auth:** Firebase (Google OAuth)
- **State:** SWR, Zustand
- **UI:** Radix UI

---

## Getting Started

### Prerequisites

- Node.js >= 18
- pnpm (install via `npm i -g pnpm`)

### Installation

1. Clone the repository:
   ```bash
   git clone git@github.com:fluke-jesadakorn/epsx.git
   cd epsx/apps/frontend
   ```

2. Install dependencies:
   ```bash
   pnpm install
   ```

3. Setup environment variables:
   - Copy `.env.example` to `.env`
   - Fill in required values (Firebase, Google OAuth, etc.)

4. Run the app:
   ```bash
   pnpm dev
   ```

---

## Environment Variables

- Firebase project credentials
- Google OAuth credentials
- Other app-specific settings (see `.env.example`)

---

## Scripts

- `pnpm dev` — Start development server
- `pnpm build` — Build for production
- `pnpm lint` — Lint code
- `pnpm format` — Format codebase
- `pnpm type-check` — Type check

---

## Contributing

- Use `pnpm` for dependencies
- Run `pnpm lint` and `pnpm format` before committing
- Follow code style enforced by Prettier and ESLint

---

## License

MIT (or specify your license)

---

## Acknowledgments

Built with [Next.js](https://nextjs.org/), [React](https://react.dev/), [Tailwind CSS](https://tailwindcss.com/), [Firebase](https://firebase.google.com/), and [Radix UI](https://www.radix-ui.com/).
