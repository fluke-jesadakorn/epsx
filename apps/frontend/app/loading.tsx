import { Skeleton } from 'antd';
import { Suspense } from 'react';

export default function Loading() {
  return (
    <Suspense
      fallback={
        <div style={{ 
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          backgroundColor: 'rgba(255, 255, 255, 0.8)',
          zIndex: 1000
        }}>
          <Skeleton 
            active
            avatar={{ shape: 'square', size: 'large' }}
            paragraph={{ rows: 4 }}
            title={false}
          />
        </div>
      }
    >
      {null}
    </Suspense>
  );
}

// TODO: Add support for different skeleton shapes (circle, rectangle, etc.)
// TODO: Implement animation options (wave, pulse, fade)
// TODO: Add theme support for dark/light mode
// TODO: Consider adding progress indicators for long operations
