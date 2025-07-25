import { Suspense } from 'react';
import { getUsers } from '@/lib/data/admin';
import { UserManagementClient } from './UserManagementClient';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';

interface UserManagementServerProps {
  searchParams?: { [key: string]: string | string[] | undefined };
}

async function UserList({ searchParams }: { searchParams?: URLSearchParams }) {
  try {
    const data = await getUsers(searchParams);
    return <UserManagementClient initialData={data} />;
  } catch (error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Error Loading Users</CardTitle>
          <CardDescription>
            Failed to load user data. Please try again later.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">
            {error instanceof Error ? error.message : 'Unknown error occurred'}
          </p>
        </CardContent>
      </Card>
    );
  }
}

function UserListSkeleton() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>
          <Skeleton className="h-6 w-48" />
        </CardTitle>
        <CardDescription>
          <Skeleton className="h-4 w-64" />
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="flex items-center space-x-4">
            <Skeleton className="h-10 w-10 rounded-full" />
            <div className="space-y-2">
              <Skeleton className="h-4 w-48" />
              <Skeleton className="h-3 w-32" />
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}

export default function UserManagementServer({ searchParams }: UserManagementServerProps) {
  const urlSearchParams = new URLSearchParams();
  
  // Convert searchParams to URLSearchParams
  if (searchParams) {
    Object.entries(searchParams).forEach(([key, value]) => {
      if (typeof value === 'string') {
        urlSearchParams.set(key, value);
      } else if (Array.isArray(value)) {
        value.forEach(v => urlSearchParams.append(key, v));
      }
    });
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">User Management</h1>
        <p className="text-muted-foreground">
          Manage users, permissions, and access controls
        </p>
      </div>

      <Suspense fallback={<UserListSkeleton />}>
        <UserList searchParams={urlSearchParams} />
      </Suspense>
    </div>
  );
}