'use client';

import { useAuth } from '@/context/auth-context-improved';
import { useState, useEffect } from 'react';

export function AuthDebugger() {
  const { user, loading, error } = useAuth();
  const [sessionInfo, setSessionInfo] = useState<any>(null);
  const [showDebug, setShowDebug] = useState(false);

  useEffect(() => {
    if (typeof window !== 'undefined') {
      // Check session cookie
      const sessionCookie = document.cookie
        .split('; ')
        .find(row => row.startsWith('__session='))
        ?.split('=')[1];
      
      setSessionInfo({
        hasSessionCookie: !!sessionCookie,
        sessionLength: sessionCookie?.length || 0,
        url: window.location.href,
        timestamp: new Date().toISOString()
      });
    }
  }, [user, loading]);

  if (!showDebug) {
    return (
      <div className="fixed bottom-4 left-4 z-50">
        <button
          onClick={() => setShowDebug(true)}
          className="bg-blue-500 text-white px-3 py-1 rounded text-xs"
        >
          Show Auth Debug
        </button>
      </div>
    );
  }

  return (
    <div className="fixed bottom-4 left-4 z-50 bg-black bg-opacity-80 text-white p-4 rounded max-w-md text-xs">
      <div className="flex justify-between items-center mb-2">
        <h3 className="font-bold">Auth Debug</h3>
        <button
          onClick={() => setShowDebug(false)}
          className="text-red-400 hover:text-red-300"
        >
          ×
        </button>
      </div>
      <div className="space-y-1">
        <div>User: {user ? user.email : 'null'}</div>
        <div>Loading: {loading ? 'true' : 'false'}</div>
        <div>Error: {error || 'none'}</div>
        <div>Session Cookie: {sessionInfo?.hasSessionCookie ? 'present' : 'missing'}</div>
        <div>Cookie Length: {sessionInfo?.sessionLength || 0}</div>
        <div>URL: {sessionInfo?.url}</div>
        <div>Time: {sessionInfo?.timestamp}</div>
      </div>
    </div>
  );
}
