/// Top navigation with privacy mode toggle + global search.

import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Search, Eye, EyeOff, Loader2 } from 'lucide-react';
import { useSettingsStore } from '@/store/settingsStore';
import { settingsService } from '@/services/settingsService';
import { toast } from '@/store/toastStore';

export function TopNav() {
  const navigate = useNavigate();
  const privacyMode = useSettingsStore((s) => s.privacyMode);
  const togglePrivacyMode = useSettingsStore((s) => s.togglePrivacyMode);
  const [query, setQuery] = useState('');
  const [saving, setSaving] = useState(false);

  async function handleTogglePrivacy() {
    setSaving(true);
    const next = !privacyMode;
    try {
      await settingsService.set('app.privacy_mode', next);
      togglePrivacyMode();
      toast.success(next ? 'Privacymodus ingeschakeld' : 'Privacymodus uitgeschakeld');
    } catch (e) {
      toast.error(`Fout: ${e}`);
    } finally {
      setSaving(false);
    }
  }

  function handleSearch(e: React.FormEvent) {
    e.preventDefault();
    if (!query.trim()) return;
    navigate(`/media?query=${encodeURIComponent(query)}`);
  }

  return (
    <header className="bg-white border-b border-surface-200 px-6 py-3 flex items-center gap-4">
      <form onSubmit={handleSearch} className="flex-1 max-w-2xl relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-surface-400" />
        <input
          type="search"
          placeholder="Zoek in media, projecten, merken..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="w-full pl-10 pr-4 py-2 rounded-lg bg-surface-100 text-sm focus:bg-white focus:outline-none focus:ring-2 focus:ring-brand-500"
        />
      </form>
      <button
        onClick={handleTogglePrivacy}
        disabled={saving}
        className={`btn ${
          privacyMode
            ? 'bg-brand-600 text-white hover:bg-brand-700'
            : 'bg-surface-100 text-surface-700 hover:bg-surface-200'
        }`}
        title={privacyMode ? 'Privacymodus aan — prive-media verborgen' : 'Privacymodus uit'}
      >
        {saving ? (
          <Loader2 className="w-4 h-4 animate-spin" />
        ) : privacyMode ? (
          <EyeOff className="w-4 h-4" />
        ) : (
          <Eye className="w-4 h-4" />
        )}
        <span>{privacyMode ? 'Privacy AAN' : 'Privacy UIT'}</span>
      </button>
    </header>
  );
}
