import { redirect } from 'next/navigation';

export default function LoginPage() {
  // Server-side redirect - happens once, no client-side loop
  redirect('/');
}
