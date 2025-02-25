import React from "react";
import { Result } from "antd";

const NotFoundPage: React.FC = () => {
  return (
    <div style={{ 
      display: 'flex',
      justifyContent: 'center',
      alignItems: 'center',
      minHeight: '100vh'
    }}>
      <Result
        status="404"
        title="404"
        subTitle="Sorry, the page you visited does not exist."
        // TODO: Add back home button functionality
        /* Future features:
           - Add interactive back button component
           - Implement animated transition effects
           - Add search box for lost pages
        */
      />
    </div>
  );
};

export default NotFoundPage;
