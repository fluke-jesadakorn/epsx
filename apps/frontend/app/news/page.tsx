'use client';

import { Suspense } from 'react';

import { SkeletonLoader } from '@/components/common/Skeleton';

const News = () => {
  return (
    <Suspense fallback={<SkeletonLoader />}>
      <div>
        <h1>News</h1>
      </div>
    </Suspense>
  );
};

export default News;
