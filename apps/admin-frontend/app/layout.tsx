import './globals.css';
import { AdminAuthWrapper } from '@/components/providers/AdminAuthWrapper';

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
        <AdminAuthWrapper>
          {children}
        </AdminAuthWrapper>
      </body>
    </html>
  );
}