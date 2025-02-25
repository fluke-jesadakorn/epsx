import ClientChatSection from "./ClientChatSection";
import { Suspense } from "react";
import { Col, Spin } from "antd";

export default async function ChatSection() {
  // Fetch initial data on the server

  return (
    <Col span={24}>
      <Suspense
        fallback={
          <div
            style={{
              height: "calc(100vh - 6rem)",
              display: "flex",
              justifyContent: "center",
              alignItems: "center",
            }}
          >
            <Spin size="large" />
          </div>
        }
      >
        <ClientChatSection />
      </Suspense>
    </Col>
  );
}

// Example API Response Format:
/* {
  "messages": [
    {
      "role": "user",
      "content": "What are the latest financial metrics for Apple?"
    }
  ]
}
*/

// TODO: Future features
// 1. Add error handling and display error messages to user
// 2. Add ability to save favorite queries
// 3. Add support for voice input
// 4. Add message reactions/feedback
// 5. Add code syntax highlighting
// 6. Add support for file attachments
// 7. Add message search functionality
// 8. Add conversation export
// 9. Add conversation title generation
// 10. Add conversation branching
