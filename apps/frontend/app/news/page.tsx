'use client';

import { Suspense } from 'react';

const News = () => {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <div>
        <h1>News</h1>
      </div>
    </Suspense>
  );
};

export default News;
