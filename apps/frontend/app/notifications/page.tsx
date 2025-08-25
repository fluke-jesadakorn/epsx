import React from 'react';
import type { Metadata } from 'next';
import { NotificationCenterClient } from './NotificationCenterClient';

export const metadata: Metadata = {
  title: 'Notifications | EPSX',
  description: 'View and manage your notifications from the EPSX analytics platform.',
};

export default function NotificationsPage() {
  return <NotificationCenterClient />;
}