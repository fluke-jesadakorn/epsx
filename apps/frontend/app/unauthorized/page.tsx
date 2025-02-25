import { Button, Result } from 'antd';
import Link from 'next/link';
export default function UnauthorizedPage() {
  // TODO: Implement auth check
  // If user is logged in, redirect to home
  // Example:
  // const session = await getSession();
  // if (session) redirect('/');

  return (
    <div className="flex items-center justify-center min-h-screen">
      <Result
        status="403"
        title="403"
        subTitle="Sorry, you are not authorized to access this page."
        extra={
          <Link href="/login">
            <Button type="primary">Login</Button>
          </Link>
        }
      />
    </div>
  );
}

export const runtime = 'edge';
