"use client";

import React from "react";
import type { CSSProperties } from "react";

interface CardSectionProps {
  style?: CSSProperties;
  className?: string;
}

const CardSection: React.FC<CardSectionProps> = ({ style, className }) => {
  return (
    <div 
      className={`min-h-[30vh] w-full flex items-center justify-center ${className || ''}`}
      style={style}
    >
      <div className="text-center space-y-4">
        <h1 className="text-4xl md:text-5xl font-bold">
          Discover the World of Stocks{" "}
          <span className="text-[#1fc7d4]">with EPSx</span>
        </h1>
        <p className="text-lg md:text-xl text-gray-600">
          Track EPS growth rankings, manage your portfolio, and unlock deeper
          insights for smarter investment decisions.
        </p>
      </div>
    </div>
  );
};

export default CardSection;
