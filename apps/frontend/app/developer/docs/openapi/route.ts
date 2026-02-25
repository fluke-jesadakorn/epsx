import { NextResponse } from 'next/server';

const BACKEND = process.env.NEXT_PUBLIC_BACKEND_URL ?? 'http://localhost:8080';

export async function GET() {
    const res = await fetch(`${BACKEND}/api-docs/openapi.json`, {
        next: { revalidate: 300 },
    });

    if (!res.ok) {
        return NextResponse.json({ error: 'Failed to fetch spec' }, { status: 502 });
    }

    const spec = await res.json() as unknown;
    return NextResponse.json(spec);
}
