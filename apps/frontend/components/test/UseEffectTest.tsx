'use client';

import { useEffect, useState } from 'react';

export function UseEffectTest() {
  console.log('✅ UseEffectTest RENDER');
  
  const [count, setCount] = useState(0);
  
  useEffect(() => {
    console.log('✅ UseEffectTest: useEffect WORKS!!!');
    setCount(1);
  }, []);
  
  return (
    <div className="fixed bottom-4 left-4 p-2 bg-green-900 text-white rounded text-xs">
      UseEffect Test: {count}
    </div>
  );
}