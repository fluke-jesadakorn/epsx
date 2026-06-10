import { Suspense } from 'react';
import { AccessDenied } from '@/components/auth/access-denied';

interface AccessDeniedPageProps {
  searchParams: {
    reason?: string;
    route?: string;
  };
}

export default function AccessDeniedPage({ searchParams }: AccessDeniedPageProps) {
  const reason = searchParams.reason ?? 'You do not have permission to access this page';
  const route = searchParams.route;
  
  const requiredPermissions = route ? [`${route.replace('/', '')}:access`] : [];
  
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <AccessDenied 
        reason={reason}
        requiredPermissions={requiredPermissions}
      />
    </Suspense>
  );
}