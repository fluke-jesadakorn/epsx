import { getBackendUrl } from '@/shared/utils/url-resolver';
import { NextResponse, type NextRequest } from 'next/server';

export async function GET(
  _req: NextRequest,
  { params }: { params: Promise<{ conv_id: string; filename: string }> }
) {
  const { conv_id, filename } = await params;
  const url = `${getBackendUrl('server')}/api/chat/files/${conv_id}/${filename}`;

  const res = await fetch(url);
  if (!res.ok) {
    return new NextResponse('Not found', { status: 404 });
  }

  const body = res.body;
  const ct = res.headers.get('content-type') ?? 'application/octet-stream';

  return new NextResponse(body, {
    headers: {
      'Content-Type': ct,
      'Cache-Control': 'private, max-age=86400',
      'X-Content-Type-Options': 'nosniff',
    },
  });
}
