"use client";
import { Layout, Typography, Button, Flex } from "antd";

const { Content } = Layout;
const { Title, Text } = Typography;

export default function ComingSoonPage() {
  return (
    <Layout className="min-h-screen bg-gray-900">
      <Content>
        <Flex
          justify="center"
          align="center"
          vertical
          gap={10}
          style={{ height: "100vh" }}
        >
          <Title className="text-white" level={1}>
            Coming Soon
          </Title>
          
          <Text className="text-white text-lg">
            We&apos;re working on something awesome!
          </Text>

          <Button type="primary" size="large">
            Notify Me
          </Button>
        </Flex>
      </Content>
    </Layout>
  );
}
