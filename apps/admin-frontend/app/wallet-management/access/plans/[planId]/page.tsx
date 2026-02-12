import { PlanEditorPage } from '@/components/access-control/plans/plan-editor-page';

interface Props {
    params: Promise<{ planId: string }>;
}

export default async function PlanDetailPage({ params }: Props) {
    const { planId } = await params;
    return <PlanEditorPage planId={planId} />;
}
