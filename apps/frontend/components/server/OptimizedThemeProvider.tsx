import { cookies } from 'next/headers';
import { ClientThemeProvider } from '../client/ClientThemeProvider';

interface OptimizedThemeProviderProps {
  children: React.ReactNode;
}

/**
 * Server-optimized theme provider that prevents hydration mismatches
 */
export async function OptimizedThemeProvider({ children }: OptimizedThemeProviderProps) {
  // Get theme from server-side cookies
  const cookieStore = cookies();
  const themeCookie = cookieStore.get('theme');
  const serverTheme = themeCookie?.value || 'system';
  
  return (
    <html lang="en" suppressHydrationWarning data-theme={serverTheme}>
      <head>
        {/* Inline critical theme script to prevent flash */}
        <script
          dangerouslySetInnerHTML={{
            __html: `
              (function() {
                const theme = localStorage.getItem('theme') || 'system';
                const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
                const isDark = theme === 'dark' || (theme === 'system' && prefersDark);
                document.documentElement.classList.toggle('dark', isDark);
              })();
            `,
          }}
        />
      </head>
      <body suppressHydrationWarning>
        <ClientThemeProvider defaultTheme={serverTheme}>
          {children}
        </ClientThemeProvider>
      </body>
    </html>
  );
}