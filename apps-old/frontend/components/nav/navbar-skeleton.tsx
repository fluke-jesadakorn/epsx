'use client';

export function NavbarSkeleton() {
  return (
    <header className="sticky top-0 z-50 border-b border-slate-200/60 bg-white/95 backdrop-blur-md dark:border-slate-800 dark:bg-slate-950/95">
      <div className="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 md:px-6">
        <div className="h-6 w-14 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse" />
        <div className="flex items-center gap-2">
          <div className="hidden md:flex items-center gap-2">
            <div className="h-7 w-16 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse" />
            <div className="h-7 w-20 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse" />
          </div>
          <div className="h-9 w-9 bg-slate-100 dark:bg-slate-800 rounded-md animate-pulse lg:hidden" />
        </div>
      </div>
    </header>
  );
}
