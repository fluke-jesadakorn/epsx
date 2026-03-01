import { NextResponse, type NextRequest } from 'next/server';

export async function GET(
  _req: NextRequest,
  { params }: { params: Promise<{ conv_id: string; filename: string }> }
) {
  const { conv_id, filename } = await params;
  const cdn = process.env.NEXT_PUBLIC_CDN_URL ?? 'https://cdn.epsx.io';
  return NextResponse.redirect(`${cdn}/chat/${conv_id}/${filename}`);
}
