"use client";

import { Form, Input, Button, Typography } from "antd";
import { MailOutlined } from "@ant-design/icons";

const { Title, Text } = Typography;

export default function TermsPage() {
  const [form] = Form.useForm();

  const handleSubmit = (values: { email: string }) => {
    console.log("Email to store:", values.email);
    // TODO: Implement actual email storage
  };

  return (
    <div style={{ maxWidth: 800, margin: "0 auto", padding: "24px" }}>
      <Title level={2}>Terms and Conditions</Title>
      
      <Text>
        <p>Last updated: {new Date().toLocaleDateString()}</p>
        
        <h3>1. Introduction</h3>
        <p>
          Welcome to our platform. By accessing or using our services, you agree
          to be bound by these terms and conditions.
        </p>

        <h3>2. Data Collection</h3>
        <p>
          We may collect and process your email address for communication
          purposes. By providing your email, you consent to this collection.
        </p>

        <h3>3. User Responsibilities</h3>
        <p>
          You are responsible for maintaining the confidentiality of your
          account information and for all activities that occur under your
          account.
        </p>
      </Text>

      <div style={{ marginTop: "32px" }}>
        <Title level={4}>Subscribe for Updates</Title>
        <Form form={form} onFinish={handleSubmit}>
          <Form.Item
            name="email"
            rules={[
              { required: true, message: "Please input your email!" },
              { type: "email", message: "Please enter a valid email address" },
            ]}
          >
            <Input
              prefix={<MailOutlined />}
              placeholder="Enter your email"
              size="large"
            />
          </Form.Item>

          <Form.Item>
            <Button type="primary" htmlType="submit" size="large">
              Subscribe
            </Button>
          </Form.Item>
        </Form>
      </div>
    </div>
  );
}