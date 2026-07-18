/// Main layout with sidebar + top nav.

import { type ReactNode } from 'react';
import { Sidebar } from '@/components/Sidebar';
import { TopNav } from '@/components/TopNav';

interface MainLayoutProps {
  children: ReactNode;
}

export function MainLayout({ children }: MainLayoutProps) {
  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar />
      <div className="flex-1 flex flex-col overflow-hidden">
        <TopNav />
        <main className="flex-1 overflow-auto bg-surface-50">
          <div className="max-w-[1600px] mx-auto px-6 py-6">{children}</div>
        </main>
      </div>
    </div>
  );
}
