import type { Metadata } from 'next';
import { redirect } from 'next/navigation';

export const metadata: Metadata = {
  title: 'Admin Access - EPSX',
  description: 'Admin features are available in the admin portal',
};

export default async function AdminRedirectPage() {
  // Redirect to the admin frontend application
  redirect('/unauthorized');
}
