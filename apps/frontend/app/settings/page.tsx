"use client";

import { Card, App, Space } from "antd";
import ProfileSettingsForm from "@/components/settings/ProfileSettingsForm";
import ProviderAuthSettings from "@/components/settings/ProviderAuthSettings";

export default function SettingsPage() {
  return (
    <App>
      <div style={{ maxWidth: 600, margin: "0 auto", padding: "24px" }}>
        <Space direction="vertical" size="large" style={{ width: "100%" }}>
          <Card title="Profile Settings">
            <ProfileSettingsForm />
          </Card>
          <Card title="Authentication Providers">
            <ProviderAuthSettings />
          </Card>
        </Space>
      </div>
    </App>
  );
}
