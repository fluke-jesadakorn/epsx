"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import {
  Form,
  Input,
  Button,
  Alert,
  Skeleton,
  Divider,
  Col,
  Row,
  Typography,
} from "antd";
import { LockOutlined } from "@ant-design/icons";

const { Title } = Typography;

// Zod schema for form validation
const resetPasswordSchema = z.object({
  password: z.string().min(8, "Password must be at least 8 characters"),
  confirmPassword: z.string(),
}).refine((data) => data.password === data.confirmPassword, {
  message: "Passwords don't match",
  path: ["confirmPassword"],
});

type ResetPasswordForm = z.infer<typeof resetPasswordSchema>;

const ResetPasswordPage = () => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState(false);
  const router = useRouter();
  const {
    handleSubmit,
    formState: { errors },
  } = useForm<ResetPasswordForm>({
    resolver: zodResolver(resetPasswordSchema),
  });

  const onSubmit = async (values: ResetPasswordForm) => {
    setLoading(true);
    setError("");

    try {
      // TODO: Implement password reset logic
      // 1. Validate reset token from URL
      // 2. Call authentication service to update password
      // 3. Handle success/error states
      await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate API call
      setSuccess(true);
      setTimeout(() => router.push("/login"), 2000);
    } catch (error) {
      setError('Password reset functionality not implemented');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Row align={"middle"} justify={"center"} style={{ minHeight: "100vh" }}>
      <Col xs={20} sm={12} md={8} lg={6}>
        <Title
          style={{
            fontSize: 24,
            fontWeight: 600,
            marginBottom: 24,
            textAlign: "center",
          }}
        >
          Reset Password
        </Title>

        {error && (
          <Alert
            message={error}
            type="error"
            showIcon
            closable
            style={{ marginBottom: 24 }}
          />
        )}

        {success && (
          <Alert
            message="Password reset successfully! Redirecting to login..."
            type="success"
            showIcon
            closable
            style={{ marginBottom: 24 }}
          />
        )}

        <Form
          name="reset-password"
          initialValues={{ remember: true }}
          onFinish={handleSubmit(onSubmit)}
          layout="vertical"
        >
          <Form.Item
            name="password"
            validateStatus={errors.password ? "error" : ""}
            help={errors.password?.message}
          >
            <Input.Password
              prefix={<LockOutlined className="text-gray-300" />}
              placeholder="New Password"
              size="large"
            />
          </Form.Item>

          <Form.Item
            name="confirmPassword"
            validateStatus={errors.confirmPassword ? "error" : ""}
            help={errors.confirmPassword?.message}
          >
            <Input.Password
              prefix={<LockOutlined className="text-gray-300" />}
              placeholder="Confirm New Password"
              size="large"
            />
          </Form.Item>

          <Form.Item>
            <Button
              type="primary"
              htmlType="submit"
              size="large"
              block
              loading={loading}
              icon={loading ? (
                <Skeleton.Avatar 
                  active 
                  size="small" 
                  shape="circle" 
                  style={{ marginRight: 8 }}
                />
              ) : null}
              // TODO: Add skeleton loading for form fields
              // TODO: Implement progress indicators
              // TODO: Add loading states for navigation
            >
              Reset Password
            </Button>
          </Form.Item>
        </Form>

        <Divider>or</Divider>

        <Col style={{ marginTop: 16, textAlign: "center" }}>
          <Button
            type="link"
            onClick={() => router.push("/login")}
          >
            Back to Login
          </Button>
        </Col>
      </Col>
    </Row>
  );
};

export default ResetPasswordPage;
