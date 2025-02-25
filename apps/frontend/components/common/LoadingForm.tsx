"use client";

import { Card, Row, Col } from "antd";

export const LoadingForm = ({ children }: { children: React.ReactNode }) => {
  return (
    <Row align={"middle"} justify={"center"} style={{ minHeight: "100vh" }}>
      <Col style={{ maxWidth: 370, width: "100%" }}>
        <Card
          style={{
            transition: "all 0.3s ease",
            textAlign: "center",
            padding: 10,
          }}
        >
          {children}
        </Card>
      </Col>
    </Row>
  );
};
