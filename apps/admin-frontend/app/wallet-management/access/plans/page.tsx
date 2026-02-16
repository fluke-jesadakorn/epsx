import dynamic from 'next/dynamic';

const PlansView = dynamic(
  () => import('@/components/access-control/plans-view').then(m => ({ default: m.PlansView })),
  { loading: () => <div className="animate-pulse h-96 rounded-2xl bg-gray-200 dark:bg-gray-700" /> }
);

export default function PlansPage() {
    return <PlansView />;
}
