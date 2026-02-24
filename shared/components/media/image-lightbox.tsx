'use client';

import { X, ZoomIn, ZoomOut } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';

interface ImageLightboxProps {
  src: string;
  alt: string;
  onClose: () => void;
}

export function ImageLightbox({ src, alt, onClose }: ImageLightboxProps) {
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const dragStart = useRef({ x: 0, y: 0 });

  const zoomIn = useCallback(() => setZoom(z => Math.min(z + 0.5, 4)), []);
  const zoomOut = useCallback(() => setZoom(z => {
    const next = Math.max(z - 0.5, 0.5);
    if (next <= 1) setPan({ x: 0, y: 0 });
    return next;
  }), []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
      if (e.key === '+' || e.key === '=') zoomIn();
      if (e.key === '-') zoomOut();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose, zoomIn, zoomOut]);

  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.stopPropagation();
    setZoom(z => {
      const next = Math.max(0.5, Math.min(4, z * (1 - e.deltaY * 0.001)));
      if (next <= 1) setPan({ x: 0, y: 0 });
      return next;
    });
  }, []);

  const handlePointerDown = (e: React.PointerEvent<HTMLImageElement>) => {
    e.stopPropagation();
    if (zoom <= 1) return;
    setIsDragging(true);
    dragStart.current = { x: e.clientX - pan.x, y: e.clientY - pan.y };
    e.currentTarget.setPointerCapture(e.pointerId);
  };

  const handlePointerMove = (e: React.PointerEvent<HTMLImageElement>) => {
    e.stopPropagation();
    if (!isDragging) return;
    setPan({ x: e.clientX - dragStart.current.x, y: e.clientY - dragStart.current.y });
  };

  const handlePointerUp = (e: React.PointerEvent<HTMLImageElement>) => {
    e.stopPropagation();
    setIsDragging(false);
    e.currentTarget.releasePointerCapture(e.pointerId);
  };

  return (
    <div
      className="fixed inset-0 z-[9999] bg-black/85 backdrop-blur-sm flex items-center justify-center overflow-hidden"
      onClick={onClose}
      onWheel={handleWheel}
    >
      <div className="absolute top-4 right-4 flex items-center gap-1.5 z-10">
        <button
          onClick={e => { e.stopPropagation(); zoomIn(); }}
          className="w-9 h-9 rounded-xl bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors"
        >
          <ZoomIn className="w-4 h-4" />
        </button>
        <button
          onClick={e => { e.stopPropagation(); zoomOut(); }}
          className="w-9 h-9 rounded-xl bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors"
        >
          <ZoomOut className="w-4 h-4" />
        </button>
        <button
          onClick={e => { e.stopPropagation(); onClose(); }}
          className="w-9 h-9 rounded-xl bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-colors ml-1"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      <div className="absolute bottom-4 inset-x-0 text-center pointer-events-none z-10">
        <span className="text-white/40 text-xs px-3 py-1.5 rounded-full bg-black/50 backdrop-blur-md">
          {Math.round(zoom * 100)}% · scroll to zoom · drag to pan · esc to close
        </span>
      </div>

      <div className="w-full h-full flex items-center justify-center">
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img
          src={src}
          alt={alt}
          draggable={false}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerCancel={handlePointerUp}
          onClick={e => e.stopPropagation()}
          style={{
            transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
            transition: isDragging ? 'none' : 'transform 0.1s ease-out',
            cursor: zoom > 1 ? (isDragging ? 'grabbing' : 'grab') : 'default',
            touchAction: 'none'
          }}
          className="max-w-[90vw] max-h-[90vh] object-contain rounded-xl select-none"
        />
      </div>
    </div>
  );
}
