/// Application constants.

export const APP_NAME = 'Realisation Manager AI';
export const APP_VERSION = '1.0.0';
export const COMPANY_NAME = 'Mariën Sanitair en Centrale Verwarming';

export const NAV_ITEMS = [
  { id: 'dashboard', label: 'Dashboard', path: '/', icon: 'LayoutDashboard' },
  { id: 'media', label: 'Mediabibliotheek', path: '/media', icon: 'Images' },
  { id: 'projects', label: 'Projecten', path: '/projects', icon: 'FolderKanban' },
  { id: 'ai', label: 'AI-analyse', path: '/ai', icon: 'BrainCircuit' },
  { id: 'seo', label: 'SEO-manager', path: '/seo', icon: 'Search' },
  { id: 'privacy', label: 'Privacycentrum', path: '/privacy', icon: 'ShieldCheck' },
  { id: 'settings', label: 'Instellingen', path: '/settings', icon: 'Settings' },
] as const;

export const KNOWN_BRANDS = [
  'Daikin', 'Vaillant', 'Bosch', 'Viessmann', 'Remeha',
  'Buderus', 'Panasonic', 'Mitsubishi', 'Geberit', 'Grohe',
  'Hansgrohe', 'Intergas', 'Atag', 'Nefit',
];

export const BUSINESS_OBJECTS = [
  'boiler', 'heat_pump', 'radiator', 'pipe', 'valve',
  'toilet', 'sink', 'shower', 'bathtub', 'bathroom',
  'ventilation', 'airco', 'water_softener', 'technical_room',
  'nameplate', 'tool',
];
