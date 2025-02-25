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
import { MailOutlined } from "@ant-design/icons";

const { Title } = Typography;

// Zod schema for form validation
const forgotPasswordSchema = z.object({
  email: z.string().email("Please enter a valid email address"),
});

type ForgotPasswordForm = z.infer<typeof forgotPasswordSchema>;

const ForgotPasswordPage = () => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState(false);
  const router = useRouter();
  // createServerClient returns a Promise, so we need to handle it properly
  const {
    handleSubmit,
    formState: { errors },
  } = useForm<ForgotPasswordForm>({
    resolver: zodResolver(forgotPasswordSchema),
  });

  const onSubmit = async (values: ForgotPasswordForm) => {
    setLoading(true);
    setError("");

    try {
      // TODO: Implement password reset logic
      // 1. Call your authentication service to trigger password reset
      // 2. Send reset email to user
      // 3. Handle success/error states
      await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate API call
      setSuccess(true);
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
          Forgot Password
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
            message="Password reset email sent! Please check your inbox."
            type="success"
            showIcon
            closable
            style={{ marginBottom: 24 }}
          />
        )}

        <Form
          name="forgot-password"
          initialValues={{ remember: true }}
          onFinish={handleSubmit(onSubmit)}
          layout="vertical"
        >
          <Form.Item
            name="email"
            validateStatus={errors.email ? "error" : ""}
            help={errors.email?.message}
          >
            <Input
              prefix={<MailOutlined className="text-gray-300" />}
              placeholder="Email"
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
              icon={
                loading ? (
                  <Skeleton.Avatar
                    active
                    size="small"
                    shape="circle"
                    style={{ marginRight: 8 }}
                  />
                ) : null
              }
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
          <Button type="link" onClick={() => router.push("/login")}>
            Back to Login
          </Button>
        </Col>
      </Col>
    </Row>
  );
};

export default ForgotPasswordPage;
