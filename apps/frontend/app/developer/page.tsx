import { redirect } from 'next/navigation';

export const dynamic = 'force-dynamic';

export default function DeveloperPage() {
  redirect('/developer/keys');
}