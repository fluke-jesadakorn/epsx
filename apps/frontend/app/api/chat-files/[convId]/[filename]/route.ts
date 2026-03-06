import { NextRequest, NextResponse } from 'next/server';

export async function GET(
  _req: NextRequest,
  { params }: { params: Promise<{ convId: string; filename: string }> }
) {
  const { convId, filename } = await params;
  const backend = process.env.BACKEND_URL ?? process.env.NEXT_PUBLIC_BACKEND_URL ?? 'http://localhost:8080';
  const url = `${backend}/api/chat/files/${convId}/${filename}`;
  try {
    const res = await fetch(url);
    if (!res.ok) return new NextResponse(null, { status: res.status });
    const data = await res.arrayBuffer();
    return new NextResponse(data, {
      headers: {
        'Content-Type': res.headers.get('Content-Type') ?? 'application/octet-stream',
        'Cache-Control': 'private, max-age=3600',
      },
    });
  } catch {
    return new NextResponse(null, { status: 404 });
  }
}
