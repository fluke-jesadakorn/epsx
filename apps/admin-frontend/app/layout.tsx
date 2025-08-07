import { ClientProviders } from '@/components/providers/ClientProviders';
import './globals.css';

export const metadata = {
  title: 'EPSX Admin',
  description: 'Administrative interface for EPSX trading platform',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <ClientProviders>
          {children}
        </ClientProviders>
      </body>
    </html>
  );
}