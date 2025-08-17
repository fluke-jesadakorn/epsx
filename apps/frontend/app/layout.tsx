import { NavigationClient } from '@/components/nav/NavigationClient';
import { getAuthUser } from '@/lib/server/auth';
import { Kanit } from 'next/font/google';
import './globals.css';

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
  const user = await getAuthUser();
  
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${kanit.variable} font-sans antialiased`}>
        <NavigationClient user={user} />
        {children}
      </body>
    </html>
  );
}
