/// Sidebar navigation.

import { NavLink } from 'react-router-dom';
import {
  LayoutDashboard, Images, FolderKanban, BrainCircuit,
  Search, ShieldCheck, Settings as SettingsIcon,
  type LucideIcon,
} from 'lucide-react';
import { APP_NAME, COMPANY_NAME } from '@/lib/constants';
import { cn } from '@/lib/utils';

interface NavItem {
  id: string;
  label: string;
  path: string;
  icon: LucideIcon;
}

const ITEMS: NavItem[] = [
  { id: 'dashboard', label: 'Dashboard', path: '/', icon: LayoutDashboard },
  { id: 'media', label: 'Mediabibliotheek', path: '/media', icon: Images },
  { id: 'projects', label: 'Projecten', path: '/projects', icon: FolderKanban },
  { id: 'ai', label: 'AI-analyse', path: '/ai', icon: BrainCircuit },
  { id: 'seo', label: 'SEO-manager', path: '/seo', icon: Search },
  { id: 'privacy', label: 'Privacycentrum', path: '/privacy', icon: ShieldCheck },
  { id: 'settings', label: 'Instellingen', path: '/settings', icon: SettingsIcon },
];

export function Sidebar() {
  return (
    <aside className="w-64 bg-surface-900 text-surface-100 flex flex-col">
      <div className="px-6 py-5 border-b border-surface-800">
        <h1 className="text-lg font-bold tracking-tight">{APP_NAME}</h1>
        <p className="text-xs text-surface-400 mt-1">{COMPANY_NAME}</p>
      </div>
      <nav className="flex-1 px-3 py-4 space-y-1 overflow-y-auto">
        {ITEMS.map((item) => {
          const Icon = item.icon;
          return (
            <NavLink
              key={item.id}
              to={item.path}
              end={item.path === '/'}
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
                  isActive
                    ? 'bg-brand-600 text-white'
                    : 'text-surface-300 hover:bg-surface-800 hover:text-white',
                )
              }
            >
              <Icon className="w-4 h-4 flex-shrink-0" />
              <span>{item.label}</span>
            </NavLink>
          );
        })}
      </nav>
      <div className="px-6 py-4 border-t border-surface-800 text-xs text-surface-400">
        <p>v1.0.0 • Local AI</p>
        <p className="mt-1">© 2026 {COMPANY_NAME}</p>
      </div>
    </aside>
  );
}
