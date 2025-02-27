"use client";

import React from "react";
import { ThemeProvider } from "./ThemeProvider";
import Navbar from "../Navbar";
import { Toaster } from "sonner";

interface ClientLayoutProps {
  children: React.ReactNode;
}

const ClientLayout: React.FC<ClientLayoutProps> = ({ children }) => {
  return (
    <ThemeProvider>
      <div className="min-h-screen flex flex-col">
        <header>
          <Navbar />
        </header>
        <main className="flex-grow">{children}</main>
        <footer className="py-6 px-4 text-center border-t">
          <p className="text-sm text-gray-600">
            © {new Date().getFullYear()} EPSx. All rights reserved.
          </p>
        </footer>
      </div>
      <Toaster richColors />
    </ThemeProvider>
  );
};

export default ClientLayout;
