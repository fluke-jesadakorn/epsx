// Cookie configuration constants (used by theme system)
export const COOKIE_NAMES = {
  THEME: '__theme',
  THEME_MODE: '__theme_mode',
} as const;

export const COOKIE_CONFIG = {
  THEME: {
    maxAge: 60 * 60 * 24 * 365, // 1 year
    httpOnly: false, // Client needs access for theme switching
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax' as const,
    path: '/',
  },
  THEME_MODE: {
    maxAge: 60 * 60 * 24 * 365, // 1 year
    httpOnly: false, // Client needs access for theme switching
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax' as const,
    path: '/',
  },
} as const;