'use client';

import { useState } from 'react';

interface Props {
  src: string;
  alt: string;
  variant?: 'hero' | 'thumb';
  caption?: string;
  onClick?: () => void;
}

export function ScreenshotImg({ src, alt, variant = 'hero', caption, onClick }: Props) {
  const [err, setErr] = useState(false);
  const [expanded, setExpanded] = useState(false);

  if (err) {
    return (
      <div className={`flex items-center justify-center text-sm text-gray-500 ${variant === 'thumb' ? 'h-20' : 'h-full'}`}>
        No screenshot
      </div>
    );
  }

  const handleClick = () => {
    if (onClick) { onClick(); return; }
    setExpanded(true);
  };

  return (
    <>
      <button type="button" onClick={handleClick} className="w-full text-left">
        <img
          src={src}
          alt={alt}
          onError={() => setErr(true)}
          className={
            variant === 'thumb'
              ? 'h-20 w-full rounded border border-gray-700 object-cover object-top hover:border-blue-500 transition-colors cursor-pointer'
              : 'h-full w-full object-cover object-top cursor-pointer'
          }
        />
        {caption && variant === 'thumb' && (
          <span className="mt-1 block truncate text-xs text-gray-500">{caption}</span>
        )}
      </button>

      {expanded && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-8"
          onClick={() => setExpanded(false)}
          onKeyDown={(e) => { if (e.key === 'Escape') { setExpanded(false); } }}
          role="button"
          tabIndex={0}
        >
          <img
            src={src}
            alt={alt}
            className="max-h-full max-w-full rounded-lg shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          />
          {caption && (
            <div className="absolute bottom-6 left-1/2 -translate-x-1/2 rounded bg-gray-900/90 px-4 py-2 text-sm text-gray-300">
              {caption}
            </div>
          )}
        </div>
      )}
    </>
  );
}
