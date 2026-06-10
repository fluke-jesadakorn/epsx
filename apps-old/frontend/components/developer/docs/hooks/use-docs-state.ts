'use client';

import { useState, useCallback } from 'react';

export function useDocsState() {
  const [activeSection, setActiveSection] = useState<string | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(true);

  const toggleSection = useCallback((id: string) => {
    setActiveSection((prev) => (prev === id ? null : id));
  }, []);

  const toggleSidebar = useCallback(() => {
    setSidebarOpen((prev) => !prev);
  }, []);

  const scrollToSection = useCallback((id: string) => {
    setActiveSection(id);
    document.querySelector(`#section-${id}`)?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }, []);

  return { activeSection, sidebarOpen, toggleSection, toggleSidebar, scrollToSection };
}
