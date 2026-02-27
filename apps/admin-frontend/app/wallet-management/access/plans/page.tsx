import dynamic from 'next/dynamic';

async function importPlansView() {
  const m = await import('@/components/access-control/plans-view');
  return { default: m.PlansView };
}

const PlansView = dynamic(
  importPlansView,
  { loading: () => <div className="animate-pulse h-96 rounded-2xl bg-gray-200 dark:bg-muted" /> }
);

export default function PlansPage() {
    return <PlansView />;
}
