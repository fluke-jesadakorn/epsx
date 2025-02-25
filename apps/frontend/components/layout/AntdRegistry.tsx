"use client";
import React from "react";
import { ConfigProvider, Row } from "antd";
import { AntdRegistry } from "@ant-design/nextjs-registry";
import { useThemeStore } from "@/lib/store/theme";
import { Layout } from "antd";
import Navbar from "../Navbar";
import { unstableSetRender } from "antd";
import { createRoot, Root } from "react-dom/client";

const { Content, Footer } = Layout;

interface AntdRegistryProps {
  children: React.ReactNode;
}

interface ContainerWithReactRoot extends HTMLElement {
  _reactRoot?: Root; // React root for the container
}

const AntdRegistryProvider: React.FC<AntdRegistryProps> = ({ children }) => {
  unstableSetRender(
    (node: React.ReactNode, container: Element | DocumentFragment) => {
      // Check if the container is an HTMLElement
      if (!(container instanceof HTMLElement)) {
        throw new Error("Container must be an HTMLElement");
      }

      const containerWithRoot = container as ContainerWithReactRoot; // Extend HTMLElement to include _reactRoot
      containerWithRoot._reactRoot ||= createRoot(containerWithRoot); // Create a React root if it doesn't exist
      const root = containerWithRoot._reactRoot!;
      root.render(node);

      return async () => {
        await new Promise((resolve) => setTimeout(resolve, 0));
        root.unmount();
      };
    }
  );

  const theme = useThemeStore((state) => state.theme);

  return (
    <AntdRegistry>
      <ConfigProvider theme={theme}>
        <Layout style={{ minHeight: "100vh" }}>
          <Navbar />
          <Content>{children}</Content>
          <Footer>
            <Row justify={"center"} align={"middle"}>
              EPSx ©2025 Powered by Next.js and Ant Design
            </Row>
          </Footer>
        </Layout>
      </ConfigProvider>
    </AntdRegistry>
  );
};

export default AntdRegistryProvider;
