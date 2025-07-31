import { redirect } from 'next/navigation';

export default function RootPage() {
  // Redirect root path to admin dashboard
  redirect('/admin');
}
