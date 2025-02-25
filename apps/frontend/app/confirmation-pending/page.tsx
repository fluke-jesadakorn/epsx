"use client";

import { Card, Typography, Button } from "antd";

export default function ConfirmationPending() {
  return (
    <Card className="max-w-md mx-auto mt-10">
      <Typography.Title level={2} className="text-center mb-4">
        Confirm Your Email
      </Typography.Title>
      <Typography.Paragraph className="mb-4">
        We&apos;ve sent a confirmation email to your inbox. Please check your email
        and click the confirmation link to activate your account.
      </Typography.Paragraph>
      <Typography.Text type="secondary">
        Didn&apos;t receive the email? Check your spam folder or
        {/* TODO: Implement resend confirmation email functionality with proper error handling */}
        {/* TODO: Add rate limiting for resend requests */}
        {/* TODO: Add loading state during resend operation */}{" "}
        <Button
          type="primary"
          onClick={() => alert("Resend confirmation email")}
        >
          Click here to resend
        </Button>
      </Typography.Text>
    </Card>
  );
}