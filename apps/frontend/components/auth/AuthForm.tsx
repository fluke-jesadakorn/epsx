import { Button, Form, Input } from "antd";
import { GoogleOutlined, GithubOutlined } from "@ant-design/icons";

export interface AuthFormProps {
  mode: 'login' | 'register';
  onSubmit: (values: { email: string; password: string }) => Promise<void>;
  onOAuthClick: (provider: 'google' | 'github') => Promise<void>;
  isSubmitting: boolean;
}

export const AuthForm: React.FC<AuthFormProps> = ({
  mode,
  onSubmit,
  onOAuthClick,
  isSubmitting
}) => {
  const [form] = Form.useForm();

  const handleSubmit = async (values: { email: string; password: string }) => {
    await onSubmit(values);
    form.resetFields();
  };

  return (
    <div className="w-full max-w-md p-8 space-y-6 bg-white rounded-lg shadow-lg">
      <h2 className="text-2xl font-bold text-center text-gray-800">
        {mode === 'login' ? 'Sign In' : 'Create Account'}
      </h2>

      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        className="space-y-4"
      >
        <Form.Item
          label="Email"
          name="email"
          rules={[
            { required: true, message: 'Please input your email!' },
            { type: 'email', message: 'Please enter a valid email!' }
          ]}
        >
          <Input size="large" placeholder="Enter your email" />
        </Form.Item>

        <Form.Item
          label="Password"
          name="password"
          rules={[
            { required: true, message: 'Please input your password!' },
            { min: 6, message: 'Password must be at least 6 characters long!' }
          ]}
        >
          <Input.Password size="large" placeholder="Enter your password" />
        </Form.Item>

        <Form.Item>
          <Button
            type="primary"
            htmlType="submit"
            size="large"
            className="w-full"
            loading={isSubmitting}
          >
            {mode === 'login' ? 'Sign In' : 'Sign Up'}
          </Button>
        </Form.Item>
      </Form>

      <div className="space-y-4">
        <div className="text-center text-gray-500">Or continue with</div>
        <div className="flex gap-4">
          <Button
            className="flex-1"
            size="large"
            icon={<GoogleOutlined />}
            onClick={() => onOAuthClick('google')}
            disabled={isSubmitting}
          >
            Google
          </Button>
          <Button
            className="flex-1"
            size="large"
            icon={<GithubOutlined />}
            onClick={() => onOAuthClick('github')}
            disabled={isSubmitting}
          >
            GitHub
          </Button>
        </div>
      </div>
    </div>
  );
};
