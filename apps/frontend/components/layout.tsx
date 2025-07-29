import { Navigation } from '@/components/nav';
import { AuthProvider } from '@/context/auth-context';
import { Toaster } from '@/components/ui/toaster';

interface LayoutProps {
  children: React.ReactNode;
}

export function Layout({ children }: LayoutProps) {
  return (
    <AuthProvider>
      <div className="min-h-screen bg-background">
        <Navigation />
        <main className="container mx-auto px-4 py-8">
          {children}
        </main>
        <Toaster />
      </div>
    </AuthProvider>
  );
}
