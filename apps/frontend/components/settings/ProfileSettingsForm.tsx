import { Form, Input, Button, Space, App } from "antd";
import { LockOutlined } from "@ant-design/icons";
import { useAuth } from "@/contexts/AuthContext";
import { useState } from "react";
import { updateProfile, sendPasswordResetEmail } from "firebase/auth";
import { auth } from "@/lib/firebase-client";

interface ProfileFormValues {
  displayName: string;
  email: string;
}

export default function ProfileSettingsForm() {
  const { user } = useAuth();
  const [loading, setLoading] = useState(false);
  const [form] = Form.useForm();
  const firebaseUser = user?.firebaseUser;

  if (!firebaseUser) {
    return null;
  }

  const { message } = App.useApp();

  const handlePasswordReset = async () => {
    if (!firebaseUser.email) return;
    
    try {
      setLoading(true);
      await sendPasswordResetEmail(auth, firebaseUser.email);
      message.success("Password reset email sent successfully");
    } catch (error) {
      message.error("Failed to send password reset email");
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (values: ProfileFormValues) => {
    try {
      setLoading(true);
      await updateProfile(firebaseUser, {
        displayName: values.displayName,
      });
      message.success("Profile updated successfully");
    } catch (error) {
      message.error("Failed to update profile");
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Form
      form={form}
      layout="vertical"
      onFinish={handleSubmit}
      initialValues={{
        displayName: firebaseUser.displayName || "",
        email: firebaseUser.email || "",
      }}
    >
      <Form.Item
        label="Display Name"
        name="displayName"
        rules={[{ required: true, message: "Please input your display name!" }]}
      >
        <Input />
      </Form.Item>

      <Form.Item label="Email" name="email">
        <Input disabled />
      </Form.Item>

      <Form.Item>
        <Space direction="vertical" style={{ width: '100%' }}>
          <Button type="primary" htmlType="submit" loading={loading} block>
            Save Changes
          </Button>
          <Button 
            icon={<LockOutlined />} 
            onClick={handlePasswordReset} 
            loading={loading} 
            block
          >
            Change Password (Send Email)
          </Button>
        </Space>
      </Form.Item>
    </Form>
  );
}
