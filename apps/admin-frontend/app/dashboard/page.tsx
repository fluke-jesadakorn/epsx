import { redirect } from 'next/navigation';

// Dashboard route that redirects to the main page
// This ensures /dashboard works as expected after login
export default function DashboardPage() {
  redirect('/');
}