'use client';

import { useState, useCallback, useMemo } from 'react';

type TokenType = 'key' | 'string' | 'number' | 'boolean' | 'null' | 'punctuation';

interface Token {
  text: string;
  type: TokenType;
}

function classify(val: string): TokenType {
  if (val.startsWith('"')) {
    return val.trimEnd().endsWith(':') ? 'key' : 'string';
  }
  if (/^-?\d/.test(val)) { return 'number'; }
  if (val === 'true' || val === 'false') { return 'boolean'; }
  if (val === 'null') { return 'null'; }
  return 'punctuation';
}

function tokenize(json: string): Token[] {
  // Simple line-by-line tokenizer using JSON.stringify output (trusted input)
  const tokens: Token[] = [];
  // eslint-disable-next-line security/detect-unsafe-regex
  const pattern = /("(?:[^"\\]|\\.)*"(?:\s*:)?|true|false|null|-?\d+(?:\.\d+)?|[{}[\],:\s]+)/g;
  let match: RegExpMatchArray | null;
  while ((match = pattern.exec(json)) !== null) {
    tokens.push({ text: match[0], type: classify(match[0]) });
  }
  return tokens;
}

const colorMap: Record<TokenType, string> = {
  key: 'text-blue-300',
  string: 'text-green-300',
  number: 'text-amber-300',
  boolean: 'text-purple-300',
  null: 'text-gray-400',
  punctuation: 'text-gray-300',
};

export function ResponseExample({ data }: { data: Record<string, unknown> }) {
  const [copied, setCopied] = useState(false);
  const json = useMemo(() => JSON.stringify(data, null, 2), [data]);
  const tokens = useMemo(() => tokenize(json), [json]);

  const copy = useCallback(() => {
    void navigator.clipboard.writeText(json);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }, [json]);

  return (
    <div className="relative">
      <button
        type="button"
        onClick={copy}
        className="absolute right-3 top-3 rounded-md bg-white/10 px-2 py-1 text-xs text-gray-300 transition-colors hover:bg-white/20"
      >
        {copied ? 'Copied' : 'Copy'}
      </button>
      <pre className="overflow-x-auto rounded-xl bg-slate-950 p-4 font-mono text-sm leading-relaxed">
        <code>
          {tokens.map((t, i) => (
            <span key={i} className={colorMap[t.type]}>{t.text}</span>
          ))}
        </code>
      </pre>
    </div>
  );
}
