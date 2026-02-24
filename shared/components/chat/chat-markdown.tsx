/* eslint-disable max-lines-per-function */
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

function renderInline(text: string): React.ReactNode[] {
  const out: React.ReactNode[] = [];
  let last = 0;
  let k = 0;
  let m: RegExpExecArray | null;
  while ((m = INLINE_RE.exec(text)) !== null) {
    if (m.index > last) out.push(text.slice(last, m.index));
    const s = m[0];
    if (m[1]) {
      // inline code
      out.push(<code key={k++} className="bg-black/15 dark:bg-white/10 rounded px-1 py-0.5 text-xs font-mono">{s.slice(1, -1)}</code>);
    } else if (m[2]) {
      // bold
      out.push(<strong key={k++} className="font-bold">{m[3]}</strong>);
    } else if (m[4]) {
      // italic
      out.push(<em key={k++} className="italic">{m[5]}</em>);
    } else if (s.startsWith('![')) {
      // image
      const alt = m[7] ?? '';
      const src = m[8] ?? '';
      if (isSafeUrl(src)) {
        // eslint-disable-next-line @next/next/no-img-element
        out.push(<img key={k++} src={src} alt={alt} className="max-w-full rounded-lg my-1 inline" />);
      } else {
        out.push(<span key={k++} className="text-red-400/80 italic text-xs">[External Image Removed]</span>);
      }
    } else if (s.startsWith('[')) {
      // link
      const lbl = m[7] ?? '';
      const href = m[8] ?? '';
      if (isSafeUrl(href)) {
        out.push(<a key={k++} href={href} target="_blank" rel="noopener noreferrer" className="underline opacity-90 hover:opacity-100">{lbl}</a>);
      } else {
        out.push(<span key={k++} className="text-red-400/80 italic text-xs" title="External links are restricted for security reasons">[External Link Removed]</span>);
      }
    }
    last = m.index + s.length;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
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

function parseBlocks(text: string): Block[] {
  const lines = text.split('\n');
  const blocks: Block[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i] ?? '';

    // Fenced code block
    const fenceMatch = /^```(\w*)/.exec(line);
    if (fenceMatch) {
      const lang = fenceMatch[1] ?? '';
      const codeLines: string[] = [];
      i++;
      while (i < lines.length && !/^```\s*$/.test(lines[i] ?? '')) {
        codeLines.push(lines[i] ?? '');
        i++;
      }
      i++; // skip closing ```
      blocks.push({ type: 'code', lang, lines: codeLines });
      continue;
    }

    // Horizontal rule
    if (/^(-{3,}|\*{3,}|_{3,})\s*$/.test(line)) {
      blocks.push({ type: 'hr', lines: [] });
      i++;
      continue;
    }

    // Heading
    const hMatch = /^(#{1,6})\s+(.+)/.exec(line);
    if (hMatch) {
      blocks.push({ type: 'h', level: (hMatch[1] ?? '').length, lines: [hMatch[2] ?? ''] });
      i++;
      continue;
    }

    // Table (need header + separator)
    if (i + 1 < lines.length && /^\|/.test(line) && /^\|[\s:]*-/.test(lines[i + 1] ?? '')) {
      const headerCells = parseTblRow(line);
      const sepLine = lines[i + 1] ?? '';
      const aligns = parseTblRow(sepLine).map(c => {
        const t = c.trim();
        if (t.startsWith(':') && t.endsWith(':')) return 'center' as const;
        if (t.endsWith(':')) return 'right' as const;
        if (t.startsWith(':')) return 'left' as const;
        return null;
      });
      const rows: string[][] = [headerCells];
      i += 2;
      while (i < lines.length && /^\|/.test(lines[i] ?? '')) {
        rows.push(parseTblRow(lines[i] ?? ''));
        i++;
      }
      blocks.push({ type: 'table', lines: [], rows, aligns });
      continue;
    }

    // Blockquote
    if (/^>\s?/.test(line)) {
      const bqLines: string[] = [];
      while (i < lines.length && /^>\s?/.test(lines[i] ?? '')) {
        bqLines.push((lines[i] ?? '').replace(/^>\s?/, ''));
        i++;
      }
      blocks.push({ type: 'bq', lines: bqLines });
      continue;
    }

    // Unordered list
    if (/^[\s]*[-*+]\s/.test(line)) {
      const items: string[][] = [];
      while (i < lines.length && /^[\s]*[-*+]\s/.test(lines[i] ?? '')) {
        items.push([(lines[i] ?? '').replace(/^[\s]*[-*+]\s/, '')]);
        i++;
      }
      blocks.push({ type: 'ul', lines: [], items });
      continue;
    }

    // Ordered list
    if (/^[\s]*\d+[.)]\s/.test(line)) {
      const items: string[][] = [];
      while (i < lines.length && /^[\s]*\d+[.)]\s/.test(lines[i] ?? '')) {
        items.push([(lines[i] ?? '').replace(/^[\s]*\d+[.)]\s/, '')]);
        i++;
      }
      blocks.push({ type: 'ol', lines: [], items });
      continue;
    }

    // Empty line
    if (line.trim() === '') {
      i++;
      continue;
    }

    // Paragraph
    const pLines: string[] = [line];
    i++;
    while (i < lines.length) {
      const nl = lines[i] ?? '';
      if (
        nl.trim() === '' ||
        /^```/.test(nl) ||
        /^#{1,6}\s/.test(nl) ||
        /^>\s?/.test(nl) ||
        /^[-*+]\s/.test(nl) ||
        /^\d+[.)]\s/.test(nl) ||
        /^(-{3,}|\*{3,}|_{3,})\s*$/.test(nl) ||
        (/^\|/.test(nl) && i + 1 < lines.length && /^\|[\s:]*-/.test(lines[i + 1] ?? ''))
      ) break;
      pLines.push(nl);
      i++;
    }
    blocks.push({ type: 'p', lines: pLines });
  }

  return blocks;
}

function parseTblRow(line: string): string[] {
  return line.replace(/^\|/, '').replace(/\|$/, '').split('|').map(c => c.trim());
}

// --- Render blocks ---

function renderBlocks(blocks: Block[]): React.ReactNode[] {
  return blocks.map((b, i) => {
    switch (b.type) {
      case 'hr':
        return <hr key={i} className="my-2 border-current/15" />;

      case 'h': {
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
        return <Tag key={i} className={cls}>{content}</Tag>;
      }

      case 'code':
        return (
          <pre key={i} className="bg-black/10 dark:bg-black/30 rounded-lg p-3 my-1.5 overflow-x-auto text-xs">
            <code className="font-mono">{b.lines.join('\n')}</code>
          </pre>
        );

      case 'bq':
        return (
          <blockquote key={i} className="border-l-2 border-current/20 pl-3 my-1.5 opacity-80 italic">
            {b.lines.map((l, j) => <p key={j} className="mb-0.5 last:mb-0">{renderInline(l)}</p>)}
          </blockquote>
        );

      case 'ul':
        return (
          <ul key={i} className="list-disc list-inside mb-1.5 space-y-0.5">
            {(b.items ?? []).map((item, j) => (
              <li key={j} className="leading-relaxed">{renderInline(item.join(' '))}</li>
            ))}
          </ul>
        );

      case 'ol':
        return (
          <ol key={i} className="list-decimal list-inside mb-1.5 space-y-0.5">
            {(b.items ?? []).map((item, j) => (
              <li key={j} className="leading-relaxed">{renderInline(item.join(' '))}</li>
            ))}
          </ol>
        );

      case 'table': {
        const rows = b.rows ?? [];
        const aligns = b.aligns ?? [];
        const header = rows[0] ?? [];
        const body = rows.slice(1);
        return (
          <div key={i} className="overflow-x-auto my-1.5">
            <table className="min-w-full text-xs border-collapse">
              <thead className="border-b border-current/15">
                <tr>
                  {header.map((c, ci) => (
                    <th key={ci} className="text-left px-2 py-1 font-semibold" style={{ textAlign: aligns[ci] ?? undefined }}>
                      {renderInline(c)}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {body.map((row, ri) => (
                  <tr key={ri}>
                    {row.map((c, ci) => (
                      <td key={ci} className="px-2 py-1 border-t border-current/8" style={{ textAlign: aligns[ci] ?? undefined }}>
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

      case 'p':
      default:
        return (
          <p key={i} className="mb-1.5 last:mb-0">
            {renderInline(b.lines.join('\n'))}
          </p>
        );
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
