import { Button, Card, Space, Typography, App } from "antd";
import { useAuth } from "@/contexts/AuthContext";
import {
  GoogleAuthProvider,
  OAuthProvider,
  linkWithPopup,
  unlink,
  EmailAuthProvider,
} from "firebase/auth";
import GoogleIcon from "@/public/icons/google";
import MicrosoftIcon from "@/public/icons/microsoft";
import { useState } from "react";

const { Text } = Typography;

const buttonStyle = {
  width: "100%",
  height: "48px",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  gap: "8px",
};

export default function ProviderAuthSettings() {
  const { user } = useAuth();
  const [loading, setLoading] = useState(false);
  const { message } = App.useApp();
  const firebaseUser = user?.firebaseUser;

  if (!firebaseUser) return null;

  const providers = firebaseUser.providerData.map(
    (provider) => provider.providerId
  );

  const handleLink = async (providerName: string) => {
    if (!firebaseUser) return;

    try {
      setLoading(true);
      let provider;

      switch (providerName) {
        case "google.com":
          provider = new GoogleAuthProvider();
          break;
        case "microsoft.com":
          provider = new OAuthProvider("microsoft.com");
          break;
        default:
          throw new Error("Unsupported provider");
      }

      await linkWithPopup(firebaseUser, provider);
      message.success(`Successfully linked ${providerName} account`);
    } catch (error: any) {
      if (error.code === "auth/provider-already-linked") {
        message.error("This provider is already linked to your account");
      } else if (error.code === "auth/credential-already-in-use") {
        message.error("This account is already linked to another user");
      } else {
        message.error("Failed to link account");
        console.error(error);
      }
    } finally {
      setLoading(false);
    }
  };

  const handleUnlink = async (providerId: string) => {
    if (!firebaseUser) return;

    try {
      setLoading(true);
      await unlink(firebaseUser, providerId);
      message.success(`Successfully unlinked ${providerId} account`);
    } catch (error) {
      message.error("Failed to unlink account");
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Space direction="vertical" size="middle" style={{ width: "100%" }}>
      <div>
        <Text strong>Email Provider</Text>
        <div style={{ marginTop: 8 }}>
          {providers.includes(EmailAuthProvider.PROVIDER_ID) ? (
            <Button
              danger
              onClick={() => handleUnlink(EmailAuthProvider.PROVIDER_ID)}
              loading={loading}
              disabled={providers.length === 1}
              style={buttonStyle}
            >
              Remove Email Authentication
            </Button>
          ) : (
            <Text type="secondary">Email authentication is required</Text>
          )}
        </div>
      </div>

      <div>
        <Text strong>Google</Text>
        <div style={{ marginTop: 8 }}>
          {providers.includes("google.com") ? (
            <Button
              danger
              onClick={() => handleUnlink("google.com")}
              loading={loading}
              disabled={providers.length === 1}
              style={buttonStyle}
            >
              Unlink Google Account
            </Button>
          ) : (
            <Button
              icon={<GoogleIcon width={32} height={32} />}
              onClick={() => handleLink("google.com")}
              loading={loading}
              style={buttonStyle}
            >
              Link Google Account
            </Button>
          )}
        </div>
      </div>

      <div>
        <Text strong>Microsoft Azure</Text>
        <div style={{ marginTop: 8 }}>
          {providers.includes("microsoft.com") ? (
            <Button
              danger
              onClick={() => handleUnlink("microsoft.com")}
              loading={loading}
              disabled={providers.length === 1}
              style={buttonStyle}
            >
              Unlink Microsoft Account
            </Button>
          ) : (
            <Button
              icon={<MicrosoftIcon width={32} height={32} />}
              onClick={() => handleLink("microsoft.com")}
              loading={loading}
              style={buttonStyle}
            >
              Link Microsoft Account
            </Button>
          )}
        </div>
      </div>
    </Space>
  );
}
