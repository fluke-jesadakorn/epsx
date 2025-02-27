import { Result } from "@/components/ui/result";
import { Button } from "@/components/ui/button";
import Link from "next/link";

export default function UnauthorizedPage() {
  return (
    <div className="min-h-[calc(100vh-4rem)] flex items-center justify-center p-4">
      <Result
        status="error"
        title="401"
        subTitle="Sorry, you are not authorized to access this page."
        extra={
          <Link href="/">
            <Button>Back Home</Button>
          </Link>
        }
      />
    </div>
  );
}
