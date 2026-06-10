'use client';

import { useState, useCallback, useMemo } from 'react';
import type { EndpointDef } from './data/endpoints';

type Lang = 'curl' | 'javascript' | 'python';

const LANGS: { id: Lang; label: string }[] = [
  { id: 'curl', label: 'cURL' },
  { id: 'javascript', label: 'JavaScript' },
  { id: 'python', label: 'Python' },
];

function generateSnippet(endpoint: EndpointDef, lang: Lang): string {
  const { method, path } = endpoint;
  const baseUrl = 'https://api.epsx.io';
  const url = `${baseUrl}${path}`;

  switch (lang) {
    case 'curl': {
      const parts = [`curl -X ${method} "${url}"`];
      parts.push('  -H "Authorization: Bearer YOUR_API_KEY"');
      if (method === 'POST') {
        parts.push('  -H "Content-Type: application/json"');
        parts.push('  -d \'{"ticker": "AAPL"}\'');
      }
      return parts.join(' \\\n');
    }
    case 'javascript': {
      const opts = [`  method: '${method}'`];
      opts.push("  headers: { 'Authorization': 'Bearer YOUR_API_KEY' }");
      if (method === 'POST') {
        opts.push("  body: JSON.stringify({ ticker: 'AAPL' })");
      }
      return `const res = await fetch('${url}', {\n${opts.join(',\n')}\n});\nconst data = await res.json();`;
    }
    case 'python': {
      const lines = ['import requests', ''];
      lines.push(`url = "${url}"`);
      lines.push('headers = {"Authorization": "Bearer YOUR_API_KEY"}');
      if (method === 'POST') {
        lines.push(`res = requests.post(url, headers=headers, json={"ticker": "AAPL"})`);
      } else if (method === 'DELETE') {
        lines.push(`res = requests.delete(url, headers=headers, params={"ticker": "AAPL"})`);
      } else {
        lines.push(`res = requests.get(url, headers=headers)`);
      }
      lines.push('data = res.json()');
      return lines.join('\n');
    }
  }
}

export function CodeSnippet({ endpoint }: { endpoint: EndpointDef }) {
  const [lang, setLang] = useState<Lang>('curl');
  const [copied, setCopied] = useState(false);
  const code = useMemo(() => generateSnippet(endpoint, lang), [endpoint, lang]);

  const copy = useCallback(() => {
    void navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }, [code]);

  return (
    <div>
      <div className="flex items-center gap-1 rounded-t-xl bg-slate-800 px-3 py-2">
        {LANGS.map((l) => (
          <button
            key={l.id}
            type="button"
            onClick={() => setLang(l.id)}
            className={`rounded-md px-2.5 py-1 text-xs font-medium transition-colors ${
              lang === l.id ? 'bg-slate-600 text-white' : 'text-gray-400 hover:text-gray-200'
            }`}
          >
            {l.label}
          </button>
        ))}
        <button
          type="button"
          onClick={copy}
          className="ml-auto rounded-md bg-white/10 px-2 py-1 text-xs text-gray-300 transition-colors hover:bg-white/20"
        >
          {copied ? 'Copied' : 'Copy'}
        </button>
      </div>
      <pre className="overflow-x-auto rounded-b-xl bg-slate-900 p-4 font-mono text-sm leading-relaxed text-gray-200">
        {code}
      </pre>
    </div>
  );
}
