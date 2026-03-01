'use client';

import {
  createNewsAction,
  updateNewsAction,
  uploadNewsImageAction,
} from '@/app/news/actions';
import type { NewsArticle } from '@/shared/api/news';
import { resolveNewsImageUrl } from '@/shared/api/news';

import {
  ArrowLeft,
  Bold,
  Code,
  Code2,
  Image as ImageIcon,
  Italic,
  Link, Minus,
  Pencil,
  Quote,
  Save,
  Upload, X,
} from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useRef, useState } from 'react';
import { toast } from 'sonner';

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

const ACTIVE_TAB_CLS = 'bg-[#7645d9]/20 text-[#7645d9]';
const INACTIVE_TAB_CLS = 'text-muted-foreground hover:text-foreground';

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

async function mdToHtml(md: string): Promise<string> {
  const { micromark } = await import('micromark');
  const { gfm, gfmHtml } = await import('micromark-extension-gfm');
  return micromark(md, { extensions: [gfm()], htmlExtensions: [gfmHtml()] });
}

function domToMd(node: Node): string {
  if (node.nodeType === Node.TEXT_NODE) { return node.textContent ?? ''; }
  if (node.nodeType !== Node.ELEMENT_NODE) { return Array.from(node.childNodes).map(domToMd).join(''); }
  const el = node as Element;
  const inner = Array.from(el.childNodes).map(domToMd).join('');
  switch (el.tagName.toLowerCase()) {
    case 'h1': return `# ${inner.trim()}\n\n`;
    case 'h2': return `## ${inner.trim()}\n\n`;
    case 'h3': return `### ${inner.trim()}\n\n`;
    case 'strong': case 'b': return `**${inner}**`;
    case 'em': case 'i': return `_${inner}_`;
    case 'code': return el.parentElement?.tagName.toLowerCase() === 'pre' ? inner : `\`${inner}\``;
    case 'pre': return `\`\`\`\n${el.textContent ?? ''}\n\`\`\`\n\n`;
    case 'blockquote': return `> ${inner.trim()}\n\n`;
    case 'a': return `[${inner}](${(el as HTMLAnchorElement).href})`;
    case 'img': return `![${(el as HTMLImageElement).alt}](${(el as HTMLImageElement).src})`;
    case 'hr': return '\n---\n\n';
    case 'br': return '\n';
    case 'ul': return `${Array.from(el.children).map((li) => `- ${domToMd(li)}\n`).join('')}\n`;
    case 'ol': return `${Array.from(el.children).map((li, i) => `${i + 1}. ${domToMd(li)}\n`).join('')}\n`;
    case 'li': return inner;
    case 'p': case 'div': return `${inner}\n\n`;
    default: return inner;
  }
}

function readMarkdown(el: HTMLElement): string {
  return domToMd(el).replace(/\n{3,}/g, '\n\n').trim();
}

function setContent(el: HTMLElement, html: string): void {
  const doc = new DOMParser().parseFromString(html !== '' ? html : '<p><br></p>', 'text/html');
  el.replaceChildren(...Array.from(doc.body.childNodes).map((n) => document.importNode(n, true)));
}

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
    if (res.success && res.data !== null) {
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
    <div className="p-4 sm:p-6 lg:p-8">
      <div className="space-y-6 w-full">
        <EditorHeader
          isEdit={isEdit}
          saving={saving}
          status={state.status}
          onStatusChange={(s) => set('status', s)}
          onSave={() => void handleSave()}
          onBack={() => router.push('/news')}
        />
        <div className="rounded-2xl bg-card border border-border/20 shadow-xl p-4 sm:p-8 space-y-6">
          <CoverImageField
            coverUrl={state.coverUrl}
            uploading={uploading}
            fileInputRef={fileInputRef}
            onClear={() => set('coverUrl', '')}
            onUrlChange={(url) => set('coverUrl', url)}
            onUpload={(e) => void handleImageUpload(e)}
          />
          <div className="h-px bg-border/20" />
          <div className="space-y-2">
            <input
              value={state.title}
              onChange={(e) => handleTitleChange(e.target.value)}
              placeholder="Article title..."
              className="w-full bg-transparent text-2xl sm:text-3xl font-bold text-foreground placeholder:text-muted-foreground/40 focus:outline-none border-b border-border/20 pb-2"
            />
            <div className="flex items-center gap-1 text-sm text-muted-foreground font-mono">
              <span className="opacity-50">epsx.io/news/</span>
              <input
                value={state.slug}
                onChange={(e) => set('slug', e.target.value)}
                className="bg-transparent focus:outline-none text-[#1fc7d4] border-b border-dashed border-[#1fc7d4]/40 min-w-[120px]"
              />
            </div>
          </div>
          <div className="h-px bg-border/20" />
          <textarea
            value={state.summary}
            onChange={(e) => set('summary', e.target.value)}
            placeholder="Short description…"
            rows={2}
            className="bg-transparent resize-none w-full text-muted-foreground text-base placeholder:text-muted-foreground/30 focus:outline-none border-b border-border/10 pb-2"
          />
          <TagChipInput tagsInput={state.tagsInput} onChange={(val) => set('tagsInput', val)} />
          <div className="h-px bg-border/20" />
          <MarkdownEditor value={state.content} onChange={(val) => set('content', val)} />
        </div>
      </div>
    </div>
  );
}

function MarkdownEditor({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  const [mode, setMode] = useState<'wysiwyg' | 'markdown'>('wysiwyg');
  const editorRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    void mdToHtml(value).then((html) => {
      if (editorRef.current !== null) { setContent(editorRef.current, html); }
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const switchToMarkdown = useCallback(() => {
    if (editorRef.current !== null) { onChange(readMarkdown(editorRef.current)); }
    setMode('markdown');
  }, [onChange]);

  const switchToWysiwyg = useCallback(() => {
    void mdToHtml(value).then((html) => {
      setMode('wysiwyg');
      requestAnimationFrame(() => {
        if (editorRef.current !== null) {
          setContent(editorRef.current, html);
          editorRef.current.focus();
        }
      });
    });
  }, [value]);

  const handleInput = useCallback(() => {
    if (editorRef.current !== null) { onChange(readMarkdown(editorRef.current)); }
  }, [onChange]);

  const execCmd = useCallback((cmd: string, val?: string) => {
    editorRef.current?.focus();
    // eslint-disable-next-line @typescript-eslint/no-deprecated
    document.execCommand(cmd, false, val);
  }, []);

  const insertMarkdown = useCallback((before: string, after: string) => {
    const ta = textareaRef.current;
    if (ta === null) { return; }
    const start = ta.selectionStart;
    const end = ta.selectionEnd;
    const selected = ta.value.slice(start, end);
    onChange(ta.value.slice(0, start) + before + selected + after + ta.value.slice(end));
    requestAnimationFrame(() => {
      if (textareaRef.current !== null) {
        textareaRef.current.selectionStart = start + before.length;
        textareaRef.current.selectionEnd = start + before.length + selected.length;
        textareaRef.current.focus();
      }
    });
  }, [onChange]);

  return (
    <div className="rounded-xl overflow-hidden border border-border/30">
      <div className="flex items-center gap-1 px-3 py-2 bg-muted/20 border-b border-border/30 flex-wrap">
        {mode === 'wysiwyg' ? <WysiwygToolbar onExec={execCmd} /> : <MarkdownToolbar onInsert={insertMarkdown} />}
        <div className="h-4 w-px bg-border/40 mx-1" />
        <button
          onClick={switchToWysiwyg}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${mode === 'wysiwyg' ? ACTIVE_TAB_CLS : INACTIVE_TAB_CLS}`}
        >
          <Pencil className="w-3 h-3" />
          Editor
        </button>
        <button
          onClick={switchToMarkdown}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${mode === 'markdown' ? ACTIVE_TAB_CLS : INACTIVE_TAB_CLS}`}
        >
          <Code2 className="w-3 h-3" />
          Markdown
        </button>
      </div>
      <div
        ref={editorRef}
        contentEditable
        suppressContentEditableWarning
        onInput={handleInput}
        className={[
          'w-full min-h-[500px] px-6 py-4 bg-background text-foreground text-base leading-relaxed focus:outline-none caret-white',
          '[&_h1]:text-3xl [&_h1]:font-bold [&_h1]:my-4',
          '[&_h2]:text-2xl [&_h2]:font-bold [&_h2]:my-3',
          '[&_h3]:text-xl [&_h3]:font-semibold [&_h3]:my-2',
          '[&_p]:mb-3 [&_p]:leading-relaxed',
          '[&_strong]:font-bold [&_b]:font-bold [&_em]:italic [&_i]:italic',
          '[&_code]:bg-muted/50 [&_code]:px-1 [&_code]:rounded [&_code]:text-[0.875em] [&_code]:font-mono',
          '[&_pre]:bg-muted/50 [&_pre]:p-3 [&_pre]:rounded-lg [&_pre]:mb-3 [&_pre]:overflow-x-auto [&_pre]:font-mono [&_pre]:text-sm',
          '[&_blockquote]:border-l-2 [&_blockquote]:border-[#7645d9] [&_blockquote]:pl-4 [&_blockquote]:text-muted-foreground [&_blockquote]:mb-3',
          '[&_ul]:list-disc [&_ul]:pl-6 [&_ul]:mb-3 [&_ol]:list-decimal [&_ol]:pl-6 [&_ol]:mb-3 [&_li]:mb-1',
          '[&_a]:text-[#7645d9] [&_a]:underline [&_a]:underline-offset-2',
          '[&_hr]:border-t [&_hr]:border-border/30 [&_hr]:my-4',
          '[&_img]:max-w-full [&_img]:rounded-lg [&_img]:my-2',
        ].join(' ')}
        style={{ display: mode === 'wysiwyg' ? 'block' : 'none' }}
      />
      <textarea
        ref={textareaRef}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Write markdown content…"
        className="w-full h-[500px] px-4 py-3 bg-background text-foreground text-sm font-mono resize-none focus:outline-none caret-white"
        style={{ display: mode === 'markdown' ? 'block' : 'none' }}
      />
    </div>
  );
}

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

interface WysiwygToolbarProps {
  onExec: (cmd: string, val?: string) => void;
}

function WysiwygToolbar({ onExec }: WysiwygToolbarProps) {
  const btn = 'p-1.5 rounded-lg hover:bg-muted/40 text-muted-foreground hover:text-foreground transition-colors';
  return (
    <>
      <button title="Bold" onClick={() => onExec('bold')} className={btn}><Bold className="w-3.5 h-3.5" /></button>
      <button title="Italic" onClick={() => onExec('italic')} className={btn}><Italic className="w-3.5 h-3.5" /></button>
      <button title="H1" onClick={() => onExec('formatBlock', 'h1')} className={`${btn} text-xs font-bold`}>H1</button>
      <button title="H2" onClick={() => onExec('formatBlock', 'h2')} className={`${btn} text-xs font-bold`}>H2</button>
      <button title="Code block" onClick={() => onExec('formatBlock', 'pre')} className={btn}><Code className="w-3.5 h-3.5" /></button>
      <button title="Quote" onClick={() => onExec('formatBlock', 'blockquote')} className={btn}><Quote className="w-3.5 h-3.5" /></button>
      <button title="HR" onClick={() => onExec('insertHorizontalRule')} className={btn}><Minus className="w-3.5 h-3.5" /></button>
    </>
  );
}

interface MarkdownToolbarProps {
  onInsert: (before: string, after: string) => void;
}

function MarkdownToolbar({ onInsert }: MarkdownToolbarProps) {
  const btn = 'p-1.5 rounded-lg hover:bg-muted/40 text-muted-foreground hover:text-foreground transition-colors';
  return (
    <>
      <button title="Bold" onClick={() => onInsert('**', '**')} className={btn}><Bold className="w-3.5 h-3.5" /></button>
      <button title="Italic" onClick={() => onInsert('_', '_')} className={btn}><Italic className="w-3.5 h-3.5" /></button>
      <button title="H1" onClick={() => onInsert('# ', '')} className={`${btn} text-xs font-bold`}>H1</button>
      <button title="H2" onClick={() => onInsert('## ', '')} className={`${btn} text-xs font-bold`}>H2</button>
      <button title="Code" onClick={() => onInsert('`', '`')} className={btn}><Code className="w-3.5 h-3.5" /></button>
      <button title="Quote" onClick={() => onInsert('> ', '')} className={btn}><Quote className="w-3.5 h-3.5" /></button>
      <button title="Link" onClick={() => onInsert('[', '](url)')} className={btn}><Link className="w-3.5 h-3.5" /></button>
      <button title="Image" onClick={() => onInsert('![', '](url)')} className={btn}><ImageIcon className="w-3.5 h-3.5" /></button>
      <button title="HR" onClick={() => onInsert('\n---\n', '')} className={btn}><Minus className="w-3.5 h-3.5" /></button>
    </>
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
    <div className="sticky top-0 z-10 backdrop-blur-md bg-background/80 border-b border-border/10 py-3 flex items-center justify-between">
      <button onClick={onBack} className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors">
        <ArrowLeft className="w-4 h-4" />
        Back to News
      </button>
      <div className="flex items-center gap-3">
        <div className="flex items-center rounded-lg border border-border/20 bg-card overflow-hidden">
          <button
            onClick={() => onStatusChange('draft')}
            className={`px-3 py-1.5 text-xs font-medium transition-colors ${status === 'draft' ? ACTIVE_TAB_CLS : INACTIVE_TAB_CLS}`}
          >
            Draft
          </button>
          <button
            onClick={() => onStatusChange('published')}
            className={`px-3 py-1.5 text-xs font-medium transition-colors ${status === 'published' ? ACTIVE_TAB_CLS : INACTIVE_TAB_CLS}`}
          >
            Published
          </button>
        </div>
        <button
          onClick={onSave}
          disabled={saving}
          className="flex items-center gap-2 px-4 py-2 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white text-sm font-semibold hover:opacity-90 disabled:opacity-60 transition-opacity"
        >
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
  const [showUrl, setShowUrl] = useState(false);
  const resolvedUrl = resolveNewsImageUrl(coverUrl) ?? coverUrl;

  return (
    <div className="space-y-2">
      {resolvedUrl !== '' && resolvedUrl !== null ? (
        <div className="relative group rounded-2xl overflow-hidden">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img src={resolvedUrl} alt="Cover" className="w-full h-40 sm:h-56 object-cover" />
          <div className="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-3">
            <button
              onClick={() => fileInputRef.current?.click()}
              disabled={uploading}
              className="px-3 py-1.5 rounded-lg bg-white/20 backdrop-blur-sm text-white text-sm font-medium hover:bg-white/30 transition-colors disabled:opacity-60"
            >
              {uploading ? 'Uploading…' : 'Change'}
            </button>
            <button
              onClick={onClear}
              className="px-3 py-1.5 rounded-lg bg-white/20 backdrop-blur-sm text-white text-sm font-medium hover:bg-white/30 transition-colors"
            >
              Remove
            </button>
          </div>
        </div>
      ) : (
        <div
          onClick={() => fileInputRef.current?.click()}
          className="w-full h-40 sm:h-56 rounded-2xl border-2 border-dashed border-border/30 bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent flex flex-col items-center justify-center gap-3 cursor-pointer hover:border-[#7645d9]/40 transition-colors"
        >
          {uploading ? (
            <p className="text-sm text-muted-foreground">Uploading…</p>
          ) : (
            <>
              <div className="p-3 rounded-full bg-muted/40">
                <Upload className="w-5 h-5 text-muted-foreground" />
              </div>
              <div className="text-center">
                <p className="text-sm font-medium text-muted-foreground">Add cover image</p>
                <p className="text-xs text-muted-foreground/60 mt-0.5">Click to upload</p>
              </div>
            </>
          )}
        </div>
      )}
      <div>
        <button
          onClick={() => setShowUrl((v) => !v)}
          className="text-xs text-muted-foreground/60 hover:text-muted-foreground transition-colors"
        >
          Or paste URL
        </button>
        {showUrl && (
          <input
            value={coverUrl}
            onChange={(e) => onUrlChange(e.target.value)}
            placeholder="https://example.com/image.jpg"
            className="mt-1.5 w-full px-3 py-2 rounded-xl bg-background border border-border/30 text-foreground text-xs focus:outline-none focus:ring-2 focus:ring-[#7645d9]/30"
          />
        )}
      </div>
      <input ref={fileInputRef} type="file" accept="image/jpeg,image/png,image/gif,image/webp" className="hidden" onChange={onUpload} />
    </div>
  );
}

interface TagChipInputProps {
  tagsInput: string;
  onChange: (val: string) => void;
}

function TagChipInput({ tagsInput, onChange }: TagChipInputProps) {
  const [inputVal, setInputVal] = useState('');
  const tags = tagsInput.split(',').map((t) => t.trim()).filter((t) => t.length > 0);

  const addTag = useCallback((val: string) => {
    const newTag = val.trim();
    if (newTag.length > 0 && !tags.includes(newTag)) {
      onChange([...tags, newTag].join(', '));
    }
    setInputVal('');
  }, [tags, onChange]);

  const removeTag = useCallback((tag: string) => {
    onChange(tags.filter((t) => t !== tag).join(', '));
  }, [tags, onChange]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addTag(inputVal);
    } else if (e.key === 'Backspace' && inputVal === '' && tags.length > 0) {
      removeTag(tags[tags.length - 1] ?? '');
    }
  }, [inputVal, tags, addTag, removeTag]);

  const handleBlur = useCallback(() => {
    if (inputVal.trim().length > 0) { addTag(inputVal); }
  }, [inputVal, addTag]);

  return (
    <div className="flex flex-wrap gap-1.5 items-center min-h-[36px] border-b border-border/10 pb-2">
      {tags.map((tag) => (
        <span key={tag} className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-[#7645d9]/15 text-[#7645d9] text-xs font-medium">
          {tag}
          <button onClick={() => removeTag(tag)} className="hover:opacity-70 transition-opacity">
            <X className="w-3 h-3" />
          </button>
        </span>
      ))}
      <input
        value={inputVal}
        onChange={(e) => setInputVal(e.target.value)}
        onKeyDown={handleKeyDown}
        onBlur={handleBlur}
        placeholder={tags.length === 0 ? 'Add tags…' : ''}
        className="bg-transparent text-sm text-foreground placeholder:text-muted-foreground/40 focus:outline-none min-w-[80px] flex-1"
      />
    </div>
  );
}
