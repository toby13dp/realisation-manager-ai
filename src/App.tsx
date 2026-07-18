import { useEffect } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { MainLayout } from '@/layouts/MainLayout';
import { Dashboard } from '@/pages/Dashboard';
import { MediaLibrary } from '@/pages/MediaLibrary';
import { Projects } from '@/pages/Projects';
import { ProjectDetail } from '@/pages/ProjectDetail';
import { AIAnalysis } from '@/pages/AIAnalysis';
import { SEOManager } from '@/pages/SEOManager';
import { PrivacyCenter } from '@/pages/PrivacyCenter';
import { Settings } from '@/pages/Settings';
import { useToast } from '@/hooks/useToast';
import { Toast } from '@/components/Toast';
import { onEvent } from '@/services/tauri';

export default function App() {
  const { toasts, dismiss } = useToast();

  // Listen for backend events
  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    onEvent<{ current: number; total: number }>('scan://progress', (p) => {
      // Handled by individual pages via their own listeners
      void p;
    }).then((u) => unlisteners.push(u));
    return () => unlisteners.forEach((u) => u());
  }, []);

  return (
    <MainLayout>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/media" element={<MediaLibrary />} />
        <Route path="/projects" element={<Projects />} />
        <Route path="/projects/:id" element={<ProjectDetail />} />
        <Route path="/ai" element={<AIAnalysis />} />
        <Route path="/seo" element={<SEOManager />} />
        <Route path="/privacy" element={<PrivacyCenter />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
      <Toast toasts={toasts} onDismiss={dismiss} />
    </MainLayout>
  );
}
