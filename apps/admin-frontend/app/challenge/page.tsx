import { Suspense } from 'react';
import { ChallengeClient } from './challenge-client';

interface Props {
    searchParams: Promise<{ from?: string }>;
}

export default async function ChallengePage({ searchParams }: Props) {
    const params = await searchParams;
    const from = typeof params.from === 'string' && params.from.startsWith('/') ? params.from : '/';

    return (
        <Suspense>
            <ChallengeClient from={from} />
        </Suspense>
    );
}
