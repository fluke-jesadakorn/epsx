'use client';

import dynamic from 'next/dynamic';

const BackgroundDecorationsComponent = dynamic(
  () => import('./BackgroundDecorations').then(mod => ({ default: mod.BackgroundDecorations })),
  { ssr: false }
);

export function BackgroundDecorationsClient() {
  return <BackgroundDecorationsComponent />;
}