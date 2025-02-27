import { Card, CardHeader, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import Link from "next/link";

export default function ConfirmationPendingPage() {
  return (
    <div className="min-h-[calc(100vh-4rem)] flex items-center justify-center p-4">
      <Card className="max-w-md w-full">
        <CardHeader>
          <h2 className="text-2xl font-bold text-center">Confirmation Pending</h2>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-center text-gray-600">
            Please check your email to complete the registration process. You will
            be able to access all features once your email is confirmed.
          </p>
          <div className="flex justify-center">
            <Link href="/login">
              <Button>Go to Login</Button>
            </Link>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
