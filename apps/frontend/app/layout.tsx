import { NavigationClient } from '@/components/nav/NavigationClient';
import { getAuthUser } from '@/lib/server/auth';
import { Kanit } from 'next/font/google';
import { type EPSXJWTPayload } from '@/lib/auth-utils';
import './globals.css';

// Convert EPSXJWTPayload to AuthUser format
function mapToAuthUser(payload: EPSXJWTPayload | null) {
  if (!payload) return null;
  
  return {
    user_id: payload.sub,
    email: payload.email,
    role: payload.role,
    permissions: payload.permissions,
    package_tier: payload.package_tier,
  };
}

const kanit = Kanit({
  subsets: ['latin'],
  weight: ['200', '300', '400', '500', '600', '700', '800'],
  display: 'swap',
  variable: '--font-kanit',
});

export const metadata = {
  title: 'EPSX - Stock Trading Platform',
  description: 'Advanced stock trading and analytics platform',
};

export default async function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const jwtPayload = await getAuthUser();
  const user = mapToAuthUser(jwtPayload);
  
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${kanit.variable} font-sans antialiased`}>
        <NavigationClient user={user} />
        {children}
      </body>
    </html>
  );
}
