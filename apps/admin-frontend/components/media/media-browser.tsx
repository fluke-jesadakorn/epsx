'use client';

import { deleteMediaAction, uploadMediaAction } from '@/app/media/actions';
import type { BucketName, FileInfo } from '@/shared/api/media';
import {
  Copy,
  ExternalLink,
  File,
  FolderOpen,
  Grid,
  Image,
  List,
  Search,
  Trash2,
  Upload,
  X,
} from 'lucide-react';
import Link from 'next/link';
import { memo, useCallback, useRef, useState } from 'react';
import { toast } from 'sonner';

// ============================================================================
// CONSTANTS
// ============================================================================

const PRIMARY_BTN = 'flex items-center gap-2 px-4 py-2 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white text-sm font-semibold hover:opacity-90 transition-opacity';
const ACTION_BTN = 'p-1.5 rounded-lg hover:bg-muted/50 text-muted-foreground hover:text-foreground transition-colors';

const IMG_EXTS = new Set(['jpg', 'jpeg', 'png', 'gif', 'webp']);

function extOf(key: string): string {
  return key.split('.').pop()?.toLowerCase() ?? '';
}

function isImage(key: string): boolean {
  return IMG_EXTS.has(extOf(key));
}

function nameOf(key: string): string {
  return key.split('/').pop() ?? key;
}

function fmtSize(bytes: number): string {
  if (bytes < 1024) { return `${bytes} B`; }
  if (bytes < 1024 * 1024) { return `${(bytes / 1024).toFixed(1)} KB`; }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function fmtDate(d: string | null): string {
  if (d === null) { return '—'; }
  return new Date(d).toLocaleDateString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}

// ============================================================================
// PROPS
// ============================================================================

interface Props {
  files: FileInfo[];
  bucket: BucketName;
  buckets: readonly BucketName[];
}

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

const DeleteModal = memo(function DeleteModal({ target, onCancel, onConfirm }: {
  target: FileInfo | null;
  onCancel: () => void;
  onConfirm: (f: FileInfo) => void;
}) {
  if (target === null) { return null; }
  return (
    <div className="fixed inset-0 bg-black/60 z-50 flex items-center justify-center p-4">
      <div className="rounded-2xl bg-card border border-border/20 shadow-2xl p-6 max-w-sm w-full space-y-4">
        <h3 className="font-bold text-foreground">Delete File?</h3>
        <p className="text-sm text-muted-foreground break-all">&ldquo;{nameOf(target.key)}&rdquo; will be permanently deleted.</p>
        <div className="flex gap-3 justify-end">
          <button onClick={onCancel} className="px-4 py-2 rounded-xl text-sm border border-border/20 hover:bg-muted/30">Cancel</button>
          <button onClick={() => onConfirm(target)} className="px-4 py-2 rounded-xl text-sm bg-red-500/10 text-red-400 border border-red-500/20 hover:bg-red-500/20">Delete</button>
        </div>
      </div>
    </div>
  );
});

const FileGridItem = memo(function FileGridItem({ file, onDelete, onCopy }: {
  file: FileInfo;
  onDelete: (f: FileInfo) => void;
  onCopy: (url: string) => void;
}) {
  const img = isImage(file.key);
  return (
    <div className="rounded-2xl bg-card border border-border/20 overflow-hidden group hover:border-border/40 transition-colors">
      <div className="aspect-square bg-gradient-to-br from-[#7645d9]/5 via-[#1fc7d4]/3 to-transparent flex items-center justify-center relative">
        {img ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img src={file.url} alt="" className="w-full h-full object-cover" loading="lazy" />
        ) : (
          <File className="w-10 h-10 text-muted-foreground/30" />
        )}
        <div className="absolute top-2 right-2 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <a href={file.url} target="_blank" rel="noreferrer" className="p-1.5 rounded-lg bg-black/50 text-white hover:bg-black/70" title="Open">
            <ExternalLink className="w-3.5 h-3.5" />
          </a>
          <button onClick={() => onCopy(file.url)} className="p-1.5 rounded-lg bg-black/50 text-white hover:bg-black/70" title="Copy URL">
            <Copy className="w-3.5 h-3.5" />
          </button>
          <button onClick={() => onDelete(file)} className="p-1.5 rounded-lg bg-red-600/70 text-white hover:bg-red-600/90" title="Delete">
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
      <div className="p-3 space-y-1">
        <p className="text-sm font-medium text-foreground truncate" title={file.key}>{nameOf(file.key)}</p>
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span>{fmtSize(file.size)}</span>
          <span>{fmtDate(file.last_modified)}</span>
        </div>
      </div>
    </div>
  );
});

const FileListItem = memo(function FileListItem({ file, onDelete, onCopy }: {
  file: FileInfo;
  onDelete: (f: FileInfo) => void;
  onCopy: (url: string) => void;
}) {
  const img = isImage(file.key);
  return (
    <div className="flex items-center gap-4 p-3 rounded-xl bg-card border border-border/20 hover:border-border/40 transition-colors group">
      <div className="shrink-0 w-10 h-10 rounded-lg overflow-hidden bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent border border-border/20 flex items-center justify-center">
        {img ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img src={file.url} alt="" className="w-full h-full object-cover" loading="lazy" />
        ) : (
          <File className="w-4 h-4 text-muted-foreground/40" />
        )}
      </div>
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-foreground truncate">{nameOf(file.key)}</p>
        <p className="text-xs text-muted-foreground/60 font-mono truncate">{file.key}</p>
      </div>
      <span className="text-xs text-muted-foreground whitespace-nowrap hidden sm:block">{fmtSize(file.size)}</span>
      <span className="text-xs text-muted-foreground whitespace-nowrap hidden md:block">{fmtDate(file.last_modified)}</span>
      <div className="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
        <a href={file.url} target="_blank" rel="noreferrer" className={ACTION_BTN} title="Open">
          <ExternalLink className="w-4 h-4" />
        </a>
        <button onClick={() => onCopy(file.url)} className={ACTION_BTN} title="Copy URL">
          <Copy className="w-4 h-4" />
        </button>
        <button onClick={() => onDelete(file)} className="p-1.5 rounded-lg hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition-colors" title="Delete">
          <Trash2 className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
});

function EmptyState({ bucket }: { bucket: BucketName }) {
  return (
    <div className="rounded-2xl bg-card border border-border/20 shadow-xl flex flex-col items-center justify-center py-20 gap-4">
      <div className="p-5 rounded-full bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent border border-border/20">
        <FolderOpen className="w-8 h-8 text-muted-foreground/40" />
      </div>
      <div className="text-center">
        <p className="font-semibold text-foreground">No files in &ldquo;{bucket}&rdquo;</p>
        <p className="text-sm text-muted-foreground mt-1">Upload files to get started.</p>
      </div>
    </div>
  );
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

// eslint-disable-next-line max-lines-per-function
export function MediaBrowser({ files: initialFiles, bucket, buckets }: Props) {
  const [localFiles, setLocalFiles] = useState<FileInfo[]>(initialFiles);
  const [deleteTarget, setDeleteTarget] = useState<FileInfo | null>(null);
  const [search, setSearch] = useState('');
  const [view, setView] = useState<'grid' | 'list'>('grid');
  const [uploading, setUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const filtered = search !== ''
    ? localFiles.filter((f) => f.key.toLowerCase().includes(search.toLowerCase()))
    : localFiles;

  const handleCopy = useCallback((url: string) => {
    void navigator.clipboard.writeText(url);
    toast.success('URL copied');
  }, []);

  const handleDelete = useCallback(async (file: FileInfo) => {
    const res = await deleteMediaAction(bucket, file.key);
    if (res.success) {
      toast.success('File deleted');
      setLocalFiles((prev) => prev.filter((f) => f.key !== file.key));
    } else {
      toast.error('Failed to delete file');
    }
    setDeleteTarget(null);
  }, [bucket]);

  const handleUpload = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file === undefined) { return; }

    setUploading(true);
    const formData = new FormData();
    formData.append('file', file);

    const res = await uploadMediaAction(bucket, formData);
    if (res.success && res.data !== undefined && res.data !== null) {
      toast.success('File uploaded');
      const newFile: FileInfo = {
        key: res.data.filename,
        url: res.data.url,
        size: res.data.size,
        last_modified: new Date().toISOString(),
      };
      setLocalFiles((prev) => [newFile, ...prev]);
    } else {
      toast.error('Upload failed');
    }
    setUploading(false);
    // Reset input
    if (fileInputRef.current !== null) { fileInputRef.current.value = ''; }
  }, [bucket]);

  const handleDeleteClick = useCallback((f: FileInfo) => setDeleteTarget(f), []);
  const handleDeleteCancel = useCallback(() => setDeleteTarget(null), []);
  const handleConfirmDelete = useCallback((f: FileInfo) => void handleDelete(f), [handleDelete]);

  return (
    <div className="p-4 sm:p-6 lg:p-8 space-y-6">
      <DeleteModal target={deleteTarget} onCancel={handleDeleteCancel} onConfirm={handleConfirmDelete} />

      {/* Hidden file input */}
      <input
        ref={fileInputRef}
        type="file"
        accept="image/jpeg,image/png,image/gif,image/webp,application/pdf"
        onChange={handleUpload}
        className="hidden"
      />

      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="h-[3px] w-8 bg-[#1fc7d4] rounded-full" />
          <Image className="w-5 h-5 text-[#1fc7d4]" />
          <h1 className="text-xl font-bold text-foreground">Media Browser</h1>
        </div>
        <button
          onClick={() => fileInputRef.current?.click()}
          disabled={uploading}
          className={PRIMARY_BTN}
        >
          <Upload className="w-4 h-4" />
          {uploading ? 'Uploading...' : 'Upload'}
        </button>
      </div>

      {/* Bucket tabs + search + view toggle */}
      <div className="flex items-center gap-2 flex-wrap">
        {buckets.map((b) => (
          <Link
            key={b}
            href={`/media?bucket=${b}`}
            className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors capitalize ${bucket === b ? 'bg-[#7645d9] text-white shadow-lg shadow-[#7645d9]/20' : 'bg-card border border-border/20 text-muted-foreground hover:text-foreground hover:border-border/40'}`}
          >
            {b}
          </Link>
        ))}

        <div className="ml-auto flex items-center gap-2">
          <div className="relative">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
            <input
              type="text"
              placeholder="Filter..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-8 pr-8 py-1.5 rounded-lg text-sm bg-card border border-border/20 text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:border-[#1fc7d4]/50 w-40"
            />
            {search !== '' && (
              <button onClick={() => setSearch('')} className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground">
                <X className="w-3.5 h-3.5" />
              </button>
            )}
          </div>
          <button
            onClick={() => setView(view === 'grid' ? 'list' : 'grid')}
            className={ACTION_BTN}
            title={view === 'grid' ? 'List view' : 'Grid view'}
          >
            {view === 'grid' ? <List className="w-4 h-4" /> : <Grid className="w-4 h-4" />}
          </button>
          <span className="text-sm text-muted-foreground">{filtered.length} {filtered.length === 1 ? 'file' : 'files'}</span>
        </div>
      </div>

      {/* Content */}
      {filtered.length === 0 ? (
        <EmptyState bucket={bucket} />
      ) : view === 'grid' ? (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
          {filtered.map((file) => (
            <FileGridItem key={file.key} file={file} onDelete={handleDeleteClick} onCopy={handleCopy} />
          ))}
        </div>
      ) : (
        <div className="space-y-2">
          {filtered.map((file) => (
            <FileListItem key={file.key} file={file} onDelete={handleDeleteClick} onCopy={handleCopy} />
          ))}
        </div>
      )}
    </div>
  );
}
