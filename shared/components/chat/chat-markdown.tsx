'use client';

import React from 'react';

function isSafeUrl(href: string): boolean {
  try {
    const url = new URL(href);
    return url.hostname.endsWith('epsx.io') || url.hostname === 'localhost';
  } catch { return false; }
}

// --- Inline parsing ---

const INLINE_RE = /(`[^`]+`)|(\*\*(.+?)\*\*)|(\*(.+?)\*)|(!?\[([^\]]*)\]\(([^)]+)\))/g;

function renderImage(m: RegExpExecArray, k: number): React.ReactNode {
  const alt = m[7];
  const src = m[8];
  if (isSafeUrl(src)) {
    // eslint-disable-next-line @next/next/no-img-element
    return <img key={k} src={src} alt={alt} className="max-w-full rounded-lg my-1 inline" />;
  }
  return <span key={k} className="text-red-400/80 italic text-xs">[External Image Removed]</span>;
}

function renderLink(m: RegExpExecArray, k: number): React.ReactNode {
  const lbl = m[7];
  const href = m[8];
  if (isSafeUrl(href)) {
    return <a key={k} href={href} target="_blank" rel="noopener noreferrer" className="underline opacity-90 hover:opacity-100">{lbl}</a>;
  }
  return <span key={k} className="text-red-400/80 italic text-xs" title="External links are restricted for security reasons">[External Link Removed]</span>;
}

function renderInline(text: string): React.ReactNode[] {
  const out: React.ReactNode[] = [];
  let last = 0;
  let k = 0;
  let m: RegExpExecArray | null;
  while ((m = INLINE_RE.exec(text)) !== null) {
    if (m.index > last) { out.push(text.slice(last, m.index)); }
    const s = m[0];
    if (m[1] !== undefined) {
      out.push(<code key={k++} className="bg-black/15 dark:bg-white/10 rounded px-1 py-0.5 text-xs font-mono">{s.slice(1, -1)}</code>);
    } else if (m[2] !== undefined) {
      out.push(<strong key={k++} className="font-bold">{m[3]}</strong>);
    } else if (m[4] !== undefined) {
      out.push(<em key={k++} className="italic">{m[5]}</em>);
    } else if (s.startsWith('![')) {
      out.push(renderImage(m, k++));
    } else if (s.startsWith('[')) {
      out.push(renderLink(m, k++));
    }
    last = m.index + s.length;
  }
  if (last < text.length) { out.push(text.slice(last)); }
  return out;
}

// --- Block parsing helpers ---

function parseTblRow(line: string): string[] {
  return line.replace(/^\|/, '').replace(/\|$/, '').split('|').map(c => c.trim());
}

function parseFencedCode(lines: string[], i: number, lang: string): { block: Block; next: number } {
  const codeLines: string[] = [];
  let j = i + 1;
  while (j < lines.length && !/^```\s*$/.test(lines[j] ?? '')) {
    codeLines.push(lines[j] ?? '');
    j++;
  }
  return { block: { type: 'code', lang, lines: codeLines }, next: j + 1 };
}

function parseTableBlock(lines: string[], i: number, line: string): { block: Block; next: number } {
  const headerCells = parseTblRow(line);
  const aligns = parseTblRow(lines[i + 1] ?? '').map(c => {
    const t = c.trim();
    if (t.startsWith(':') && t.endsWith(':')) { return 'center' as const; }
    if (t.endsWith(':')) { return 'right' as const; }
    if (t.startsWith(':')) { return 'left' as const; }
    return null;
  });
  const rows: string[][] = [headerCells];
  let j = i + 2;
  while (j < lines.length && /^\|/.test(lines[j] ?? '')) {
    rows.push(parseTblRow(lines[j] ?? ''));
    j++;
  }
  return { block: { type: 'table', lines: [], rows, aligns }, next: j };
}

function parseListBlock(lines: string[], i: number, type: 'ul' | 'ol'): { block: Block; next: number } {
  const pattern = type === 'ul' ? /^[\s]*[-*+]\s/ : /^[\s]*\d+[.)]\s/;
  const items: string[][] = [];
  let j = i;
  while (j < lines.length && pattern.test(lines[j] ?? '')) {
    items.push([(lines[j] ?? '').replace(pattern, '')]);
    j++;
  }
  return { block: { type, lines: [], items }, next: j };
}

function isBlockBoundary(nl: string, lines: string[], idx: number): boolean {
  if (nl.trim() === '') { return true; }
  if (/^```/.test(nl)) { return true; }
  if (/^#{1,6}\s/.test(nl)) { return true; }
  if (/^>\s?/.test(nl)) { return true; }
  if (/^[-*+]\s/.test(nl)) { return true; }
  if (/^\d+[.)]\s/.test(nl)) { return true; }
  if (/^(-{3,}|\*{3,}|_{3,})\s*$/.test(nl)) { return true; }
  if (/^\|/.test(nl) && idx + 1 < lines.length && /^\|[\s:]*-/.test(lines[idx + 1] ?? '')) { return true; }
  return false;
}

function parseParagraph(lines: string[], i: number, line: string): { block: Block; next: number } {
  const pLines: string[] = [line];
  let j = i + 1;
  while (j < lines.length) {
    const nl = lines[j] ?? '';
    if (isBlockBoundary(nl, lines, j)) { break; }
    pLines.push(nl);
    j++;
  }
  return { block: { type: 'p', lines: pLines }, next: j };
}

// --- Block parsing ---

interface Block {
  type: 'p' | 'h' | 'code' | 'bq' | 'ul' | 'ol' | 'table' | 'hr';
  level?: number;
  lang?: string;
  lines: string[];
  items?: string[][];
  rows?: string[][];
  aligns?: ('left' | 'right' | 'center' | null)[];
}

function parseBlockquote(lines: string[], i: number): { block: Block; next: number } {
  const bqLines: string[] = [];
  let j = i;
  while (j < lines.length && /^>\s?/.test(lines[j] ?? '')) {
    bqLines.push((lines[j] ?? '').replace(/^>\s?/, ''));
    j++;
  }
  return { block: { type: 'bq', lines: bqLines }, next: j };
}

function parseLinePrefix(lines: string[], i: number, line: string): { block: Block; next: number } | null {
  const fenceMatch = /^```(\w*)/.exec(line);
  if (fenceMatch !== null) { return parseFencedCode(lines, i, fenceMatch[1]); }

  if (/^(-{3,}|\*{3,}|_{3,})\s*$/.test(line)) {
    return { block: { type: 'hr', lines: [] }, next: i + 1 };
  }

  const hMatch = /^(#{1,6})\s+(.+)/.exec(line);
  if (hMatch !== null) {
    return { block: { type: 'h', level: hMatch[1].length, lines: [hMatch[2]] }, next: i + 1 };
  }

  return null;
}

function parseLine(lines: string[], i: number): { block: Block; next: number } | null {
  const line = lines[i] ?? '';
  if (line.trim() === '') { return null; }

  const prefix = parseLinePrefix(lines, i, line);
  if (prefix !== null) { return prefix; }

  if (i + 1 < lines.length && /^\|/.test(line) && /^\|[\s:]*-/.test(lines[i + 1] ?? '')) {
    return parseTableBlock(lines, i, line);
  }

  if (/^>\s?/.test(line)) { return parseBlockquote(lines, i); }
  if (/^[\s]*[-*+]\s/.test(line)) { return parseListBlock(lines, i, 'ul'); }
  if (/^[\s]*\d+[.)]\s/.test(line)) { return parseListBlock(lines, i, 'ol'); }

  return parseParagraph(lines, i, line);
}

function parseBlocks(text: string): Block[] {
  const lines = text.split('\n');
  const blocks: Block[] = [];
  let i = 0;

  while (i < lines.length) {
    const result = parseLine(lines, i);
    if (result === null) { i++; continue; }
    blocks.push(result.block);
    i = result.next;
  }

  return blocks;
}

// --- Render block helpers ---

function blockKey(b: Block): string {
  return b.type + b.lines.slice(0, 2).join('').slice(0, 30);
}

function renderHr(b: Block): React.ReactNode {
  return <hr key={blockKey(b)} className="my-2 border-current/15" />;
}

function renderHeading(b: Block): React.ReactNode {
  const content = renderInline(b.lines[0] ?? '');
  const lvl = Math.min(b.level ?? 1, 6);
  const cls = lvl === 1
    ? 'text-lg font-bold mt-3 mb-1 first:mt-0'
    : lvl === 2
      ? 'text-base font-bold mt-2.5 mb-1 first:mt-0'
      : lvl === 3
        ? 'text-sm font-bold mt-2 mb-0.5 first:mt-0'
        : 'text-sm font-semibold mt-1.5 mb-0.5 first:mt-0';
  const Tag = `h${lvl}` as keyof React.JSX.IntrinsicElements;
  return <Tag key={blockKey(b)} className={cls}>{content}</Tag>;
}

function renderCode(b: Block): React.ReactNode {
  return (
    <pre key={blockKey(b)} className="bg-black/10 dark:bg-black/30 rounded-lg p-3 my-1.5 overflow-x-auto text-xs">
      <code className="font-mono">{b.lines.join('\n')}</code>
    </pre>
  );
}

function renderBlockquote(b: Block): React.ReactNode {
  return (
    <blockquote key={blockKey(b)} className="border-l-2 border-current/20 pl-3 my-1.5 opacity-80 italic">
      {b.lines.map(l => <p key={l} className="mb-0.5 last:mb-0">{renderInline(l)}</p>)}
    </blockquote>
  );
}

function renderList(b: Block): React.ReactNode {
  const items = b.items ?? [];
  if (b.type === 'ol') {
    return (
      <ol key={blockKey(b)} className="list-decimal list-inside mb-1.5 space-y-0.5">
        {items.map(item => (
          <li key={item[0] ?? ''} className="leading-relaxed">{renderInline(item.join(' '))}</li>
        ))}
      </ol>
    );
  }
  return (
    <ul key={blockKey(b)} className="list-disc list-inside mb-1.5 space-y-0.5">
      {items.map(item => (
        <li key={item[0] ?? ''} className="leading-relaxed">{renderInline(item.join(' '))}</li>
      ))}
    </ul>
  );
}

function renderTable(b: Block): React.ReactNode {
  const rows = b.rows ?? [];
  const aligns = b.aligns ?? [];
  const header = rows[0] ?? [];
  const body = rows.slice(1);
  return (
    <div key={blockKey(b)} className="overflow-x-auto my-1.5">
      <table className="min-w-full text-xs border-collapse">
        <thead className="border-b border-current/15">
          <tr>
            {header.map(c => (
              <th key={c} className="text-left px-2 py-1 font-semibold" style={{ textAlign: aligns[header.indexOf(c)] ?? undefined }}>
                {renderInline(c)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {body.map(row => (
            <tr key={row.join('|')}>
              {row.map(c => (
                <td key={`td-${c}`} className="px-2 py-1 border-t border-current/8" style={{ textAlign: aligns[row.indexOf(c)] ?? undefined }}>
                  {renderInline(c)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function renderParagraph(b: Block): React.ReactNode {
  return (
    <p key={blockKey(b)} className="mb-1.5 last:mb-0">
      {renderInline(b.lines.join('\n'))}
    </p>
  );
}

// --- Render blocks ---

function renderBlocks(blocks: Block[]): React.ReactNode[] {
  return blocks.map(b => {
    switch (b.type) {
      case 'hr': return renderHr(b);
      case 'h': return renderHeading(b);
      case 'code': return renderCode(b);
      case 'bq': return renderBlockquote(b);
      case 'ul': return renderList(b);
      case 'ol': return renderList(b);
      case 'table': return renderTable(b);
      case 'p':
      default: return renderParagraph(b);
    }
  });
}

export function ChatMarkdown({ text }: { text: string }) {
  const blocks = parseBlocks(text);
  return (
    <div className="break-words [&>*:first-child]:mt-0">
      {renderBlocks(blocks)}
    </div>
  );
}
