import { getInitialNotificationsAction } from '@/app/actions/notifications';
import NotificationsClient from './notifications-client';

interface NotificationsPageProps {
  searchParams: Promise<{
    page?: string;
    type?: string;
    priority?: string;
    id?: string;
  }>;
}

export default async function NotificationsPage({ searchParams }: NotificationsPageProps) {
  const params = await searchParams;
  const focusId = params.id;

  const initialData = await getInitialNotificationsAction({
    page: parseInt(params.page || '1'),
    type: params.type,
    priority: params.priority,
  });

  return <NotificationsClient initialData={initialData} focusId={focusId} />;
}
