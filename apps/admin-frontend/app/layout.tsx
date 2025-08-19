import './globals.css';
import { AdminAuthWrapper } from '@/components/providers/AdminAuthWrapper';
import { Toaster } from 'react-hot-toast';

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
        <Toaster
          position="top-right"
          reverseOrder={false}
          gutter={8}
          containerStyle={{
            top: 20,
            right: 20,
          }}
          toastOptions={{
            duration: 4000,
            style: {
              borderRadius: '8px',
              background: '#363636',
              color: '#fff',
            },
            success: {
              iconTheme: {
                primary: '#10b981',
                secondary: '#fff',
              },
            },
            error: {
              iconTheme: {
                primary: '#ef4444',
                secondary: '#fff',
              },
            },
          }}
        />
      </body>
    </html>
  );
}