"use client";

import React, { useState } from "react";
import { Card, Row, Col, Typography, Flex, Pagination } from "antd";
import type { CSSProperties } from "react";
import type { EpsGrowthData } from "@/types/epsGrowthRanking";
import { fetchEpsGrowthRanking } from "@/app/actions/stockData";

interface ClientEpsCardSectionProps {
  style?: CSSProperties;
  initialData: EpsGrowthData[];
  initialTotal: number;
}

const { Title, Text } = Typography;

/**
 * Client-side EPS card section with pagination
 *
 * Future Features:
 * - Add market filter dropdown
 * - Add date range picker for last_report_date
 * - Add minimum/maximum EPS filters
 * - Add company name search
 * - Add data export to CSV/Excel
 * - Add chart visualization for EPS growth trends
 * - Add comparison feature for selected companies
 * - Add real-time data updates
 * - Add performance metrics (e.g., price change %)
 * - Add technical indicators
 * - Add mobile-optimized view
 */
export default function ClientEpsCardSection({
  style,
  initialData,
  initialTotal,
}: ClientEpsCardSectionProps) {
  const [data, setData] = useState<EpsGrowthData[]>(initialData || []);
  const [total, setTotal] = useState(initialTotal || 0);
  const [currentPage, setCurrentPage] = useState(1);
  const [loading, setLoading] = useState(false);
  const pageSize = 3;

  const handlePageChange = async (page: number) => {
    try {
      setLoading(true);
      setCurrentPage(page);
      const skip = (page - 1) * pageSize;
      const response = await fetchEpsGrowthRanking({ limit: pageSize, skip });
      if (response?.data && Array.isArray(response.data)) {
        setData(response.data);
        setTotal(response.metadata?.total || 0);
      } else {
        console.error("Invalid response format:", response);
        setData([]);
        setTotal(0);
      }
    } catch (error) {
      console.error("Failed to fetch EPS growth ranking:", error);
      setData([]);
      setTotal(0);
    } finally {
      setLoading(false);
    }
  };

  const getMarketColor = (marketCode: string | undefined) => {
    switch (marketCode) {
      case "TYO":
        return "#1890ff"; // Blue
      case "BOM":
        return "#52c41a"; // Green
      case "OTC":
        return "#722ed1"; // Purple
      default:
        return "#d9d9d9"; // Grey
    }
  };

  const renderEmptyState = () => (
    <Row gutter={[16, 16]} style={{ width: "100%", ...style }}>
      <Col span={24}>
        <Card>
          <Text type="warning">No EPS growth ranking data available.</Text>
        </Card>
      </Col>
    </Row>
  );

  if (!Array.isArray(data) || data.length === 0) {
    return renderEmptyState();
  }

  return (
    <Flex vertical gap="middle" style={{ width: "100%", ...style }}>
      <Row gutter={[16, 16]} style={{ width: "100%" }}>
        {data.map((item, index) => {
          // Skip rendering if item or required properties are missing
          if (!item?.symbol || !item?.company_name) {
            console.error("Missing required data for item:", item);
            return null;
          }

          return (
            <Col
              key={item.symbol || index}
              xs={24}
              sm={24}
              md={8}
              lg={8}
              xl={8}
            >
              <Card
                loading={loading}
                style={{
                  minHeight: 300,
                }}
                title={
                  <Flex justify="space-between" align="center">
                    <Text style={{ color: getMarketColor(item.market_code) }}>
                      {item.market_code || "N/A"}
                    </Text>
                    {index < 3 && (
                      <Text
                        style={{
                          fontWeight: "bold",
                          color:
                            index === 0
                              ? "#FFD700" // Gold
                              : index === 1
                              ? "#C0C0C0" // Silver
                              : "#CD7F32", // Bronze
                        }}
                      >
                        #{index + 1}
                      </Text>
                    )}
                  </Flex>
                }
              >
                <Flex vertical gap="small">
                  <Title level={4} style={{ margin: 0 }}>
                    {item.company_name || "Unknown Company"}
                  </Title>
                  <Text>Symbol: {item.symbol || "N/A"}</Text>
                  <Text>
                    EPS:{" "}
                    {typeof item.eps_diluted === "number"
                      ? item.eps_diluted.toFixed(2)
                      : "N/A"}
                  </Text>
                  <Text
                    style={{
                      color:
                        (item.eps_growth || 0) >= 0 ? "#52c41a" : "#ff4d4f",
                      fontWeight: "bold",
                    }}
                  >
                    Growth:{" "}
                    {typeof item.eps_growth === "number"
                      ? item.eps_growth.toFixed(2)
                      : "0"}
                    %
                  </Text>
                  <Text type="secondary">
                    Last Report:{" "}
                    {item.report_date
                      ? new Date(item.report_date).toLocaleDateString()
                      : "N/A"}
                  </Text>
                </Flex>
              </Card>
            </Col>
          );
        })}
      </Row>
    </Flex>
  );
}
