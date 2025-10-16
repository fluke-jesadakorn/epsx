import { redirect } from 'next/navigation';

export default function SignInPage() {
  // Server-side redirect - happens once, no client-side loop
  redirect('/');
}
