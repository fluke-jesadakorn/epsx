import StockRankTable from "@/components/home/StockRankTable";
import HeroSection from "@/components/home/HeroSection";
import ClientEpsCardSection from "@/components/home/ClientEpsCardSection";
import ChatSection from "@/components/home/ChatSection";
import { Col, Row } from "antd";
import { fetchEpsGrowthRanking } from "@/app/actions/stockData";
import { Suspense } from "react";
import { Spin } from "antd";

/**
 * Home page component that displays various sections including EPS growth ranking
 *
 * Future Features:
 * - Add error boundary component for better error handling
 * - Add error reporting to external service
 * - Add data prefetching for improved performance
 * - Add SEO optimization
 * - Add skeleton loading UI
 * - Add market filter functionality
 * - Add date range filtering
 * - Add sorting options
 * - Add responsive design improvements
 * - Add data export functionality
 */
async function getInitialEpsData() {
  try {
    const response = await fetchEpsGrowthRanking({ limit: 3, skip: 0 });

    // Validate response structure and data
    if (!response?.data || !Array.isArray(response.data)) {
      console.error("Invalid EPS data structure:", response);
      return {
        data: [],
        total: 0,
      };
    }

    // Filter out any invalid items
    const validData = response.data.filter(
      (item) =>
        item &&
        typeof item === "object" &&
        "symbol" in item &&
        "company_name" in item
    );

    return {
      data: validData,
      total: response.metadata?.total || validData.length,
    };
  } catch (error) {
    console.error("Failed to fetch initial EPS growth ranking:", error);
    return {
      data: [],
      total: 0,
    };
  }
}

export default async function HomeView() {
  const { data: initialData, total: initialTotal } = await getInitialEpsData();

  return (
    <Row justify={"center"} align={"middle"} style={{ minHeight: "100vh" }}>
      <Col span={20}>
        <Row
          gutter={[20, 40]}
          style={{ margin: 0, padding: "24px 0" }}
          justify={"center"}
          align={"middle"}
        >
          <Col span={24}>
            <Suspense fallback={<Spin size="large" />}>
              <HeroSection style={{ minHeight: "400px" }} />
            </Suspense>
          </Col>
          <Col span={24}>
            <Suspense fallback={<Spin size="large" />}>
              <ClientEpsCardSection
                style={{ minHeight: "300px" }}
                initialData={initialData}
                initialTotal={initialTotal}
              />
            </Suspense>
          </Col>
          <Col span={24}>
            <Suspense fallback={<Spin size="large" />}>
              <StockRankTable style={{ minHeight: "500px" }} />
            </Suspense>
          </Col>
          <Col span={24}>
            <Suspense fallback={<Spin size="large" />}>
              <ChatSection />
            </Suspense>
          </Col>
        </Row>
      </Col>
    </Row>
  );
}
