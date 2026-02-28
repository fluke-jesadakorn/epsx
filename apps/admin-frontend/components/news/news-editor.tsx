'use client';

import dynamic from 'next/dynamic';
import { useCallback, useRef, useState } from 'react';
import { useRouter } from 'next/navigation';
import { toast } from 'sonner';
import { Save, ArrowLeft, Upload, X, Image as ImageIcon } from 'lucide-react';
import type { NewsArticle } from '@/shared/api/news';
import {
  createNewsAction,
  updateNewsAction,
  uploadNewsImageAction,
} from '@/app/news/actions';

const MDEditor = dynamic(() => import('@uiw/react-md-editor/nohighlight'), { ssr: false });

interface Props {
  article?: NewsArticle | null;
}

function slugify(title: string): string {
  return title.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
}

interface EditorState {
  title: string;
  slug: string;
  summary: string;
  content: string;
  coverUrl: string;
  tagsInput: string;
  status: 'draft' | 'published';
}

const EMPTY_STATE: EditorState = { title: '', slug: '', summary: '', content: '', coverUrl: '', tagsInput: '', status: 'draft' };

function initState(a: NewsArticle | null | undefined): EditorState {
  if (a === null || a === undefined) { return EMPTY_STATE; }
  return {
    title: a.title,
    slug: a.slug,
    summary: a.summary ?? '',
    content: a.content,
    coverUrl: a.cover_image_url ?? '',
    tagsInput: a.tags.join(', '),
    status: a.status,
  };
}

const inputCls = 'w-full px-3 py-2 rounded-xl bg-background border border-border/30 text-foreground text-sm focus:outline-none focus:ring-2 focus:ring-[#7645d9]/30';

export function NewsEditor({ article }: Props) {
  const router = useRouter();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const isEdit = article !== null && article !== undefined;

  const [state, setState] = useState<EditorState>(() => initState(article));
  const [saving, setSaving] = useState(false);
  const [uploading, setUploading] = useState(false);

  const set = useCallback(<K extends keyof EditorState>(key: K, val: EditorState[K]) => {
    setState((prev) => ({ ...prev, [key]: val }));
  }, []);

  const handleTitleChange = useCallback((val: string) => {
    setState((prev) => ({ ...prev, title: val, slug: isEdit ? prev.slug : slugify(val) }));
  }, [isEdit]);

  const handleImageUpload = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file === undefined) { return; }
    setUploading(true);
    const formData = new FormData();
    formData.append('file', file);
    const res = await uploadNewsImageAction(formData);
    if (res.success) {
      set('coverUrl', res.data.url);
      toast.success('Image uploaded');
    } else {
      toast.error('Image upload failed');
    }
    setUploading(false);
    if (fileInputRef.current !== null) { fileInputRef.current.value = ''; }
  }, [set]);

  const handleSave = useCallback(async () => {
    if (state.title.trim() === '') { toast.error('Title is required'); return; }
    if (state.content.trim() === '') { toast.error('Content is required'); return; }
    setSaving(true);
    const tags = state.tagsInput.split(',').map((t) => t.trim()).filter((t) => t.length > 0);
    const data = {
      title: state.title,
      content: state.content,
      summary: state.summary !== '' ? state.summary : undefined,
      cover_image_url: state.coverUrl !== '' ? state.coverUrl : undefined,
      tags,
      status: state.status,
    };
    const res = article !== null && article !== undefined
      ? await updateNewsAction(article.id, { ...data, slug: state.slug })
      : await createNewsAction(data);
    if (res.success) {
      toast.success(isEdit ? 'Article updated' : 'Article created');
      router.push('/news');
    } else {
      toast.error('Failed to save article');
      setSaving(false);
    }
  }, [state, isEdit, article, router]);

  return (
    <div className="space-y-6 max-w-5xl">
      <EditorHeader
        isEdit={isEdit}
        saving={saving}
        status={state.status}
        onStatusChange={(s) => set('status', s)}
        onSave={() => void handleSave()}
        onBack={() => router.push('/news')}
      />
      <div className="rounded-2xl bg-card border border-border/20 shadow-xl p-6 space-y-5">
        <Field label="Title *">
          <input value={state.title} onChange={(e) => handleTitleChange(e.target.value)} placeholder="Article title" className={inputCls} />
        </Field>
        <Field label="Slug">
          <input value={state.slug} onChange={(e) => set('slug', e.target.value)} placeholder="url-friendly-slug" className={`${inputCls} font-mono text-muted-foreground`} />
        </Field>
        <Field label="Summary">
          <textarea value={state.summary} onChange={(e) => set('summary', e.target.value)} placeholder="Short description…" rows={2} className={`${inputCls} resize-none`} />
        </Field>
        <CoverImageField
          coverUrl={state.coverUrl}
          uploading={uploading}
          fileInputRef={fileInputRef}
          onClear={() => set('coverUrl', '')}
          onUrlChange={(url) => set('coverUrl', url)}
          onUpload={(e) => void handleImageUpload(e)}
        />
        <Field label="Tags">
          <input value={state.tagsInput} onChange={(e) => set('tagsInput', e.target.value)} placeholder="update, analytics (comma-separated)" className={inputCls} />
        </Field>
        <Field label="Content *">
          <div data-color-mode="dark" className="rounded-xl overflow-hidden border border-border/30">
            <MDEditor value={state.content} onChange={(val) => set('content', val ?? '')} height={500} preview="live" />
          </div>
        </Field>
      </div>
    </div>
  );
}

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <label className="text-sm font-medium text-foreground mb-1.5 block">{label}</label>
      {children}
    </div>
  );
}

interface EditorHeaderProps {
  isEdit: boolean;
  saving: boolean;
  status: 'draft' | 'published';
  onStatusChange: (s: 'draft' | 'published') => void;
  onSave: () => void;
  onBack: () => void;
}

function EditorHeader({ isEdit, saving, status, onStatusChange, onSave, onBack }: EditorHeaderProps) {
  return (
    <div className="flex items-center justify-between">
      <button onClick={onBack} className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors">
        <ArrowLeft className="w-4 h-4" />
        Back to News
      </button>
      <div className="flex items-center gap-3">
        <select value={status} onChange={(e) => onStatusChange(e.target.value as 'draft' | 'published')} className="px-3 py-1.5 rounded-lg text-sm bg-card border border-border/20 text-foreground">
          <option value="draft">Draft</option>
          <option value="published">Published</option>
        </select>
        <button onClick={onSave} disabled={saving} className="flex items-center gap-2 px-4 py-2 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white text-sm font-semibold hover:opacity-90 disabled:opacity-60">
          <Save className="w-4 h-4" />
          {saving ? 'Saving…' : isEdit ? 'Update' : 'Create'}
        </button>
      </div>
    </div>
  );
}

interface CoverImageFieldProps {
  coverUrl: string;
  uploading: boolean;
  fileInputRef: React.RefObject<HTMLInputElement | null>;
  onClear: () => void;
  onUrlChange: (url: string) => void;
  onUpload: (e: React.ChangeEvent<HTMLInputElement>) => void;
}

function CoverImageField({ coverUrl, uploading, fileInputRef, onClear, onUrlChange, onUpload }: CoverImageFieldProps) {
  return (
    <Field label="Cover Image">
      <div className="flex items-center gap-3">
        {coverUrl !== '' ? (
          <div className="relative">
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img src={coverUrl} alt="Cover" className="h-20 w-32 object-cover rounded-lg border border-border/20" />
            <button onClick={onClear} className="absolute -top-1.5 -right-1.5 p-0.5 rounded-full bg-card border border-border/20 text-muted-foreground hover:text-foreground">
              <X className="w-3 h-3" />
            </button>
          </div>
        ) : (
          <div className="h-20 w-32 rounded-lg border border-dashed border-border/40 flex items-center justify-center bg-muted/20">
            <ImageIcon className="w-6 h-6 text-muted-foreground/50" />
          </div>
        )}
        <div className="flex flex-col gap-2">
          <button onClick={() => fileInputRef.current?.click()} disabled={uploading} className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm border border-border/20 bg-card hover:bg-muted/30 text-muted-foreground hover:text-foreground transition-colors disabled:opacity-60">
            <Upload className="w-3.5 h-3.5" />
            {uploading ? 'Uploading…' : 'Upload Image'}
          </button>
          <input value={coverUrl} onChange={(e) => onUrlChange(e.target.value)} placeholder="Or paste image URL" className="px-3 py-1.5 rounded-lg bg-background border border-border/30 text-foreground text-xs focus:outline-none focus:ring-1 focus:ring-[#7645d9]/30" />
        </div>
        <input ref={fileInputRef} type="file" accept="image/jpeg,image/png,image/gif,image/webp" className="hidden" onChange={onUpload} />
      </div>
    </Field>
  );
}
