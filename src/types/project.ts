/// Project domain types.

export type ProjectType =
  | 'sanitary_bathroom'
  | 'cv_boiler'
  | 'heat_pump'
  | 'radiator'
  | 'ventilation'
  | 'airco'
  | 'water_softener'
  | 'technical_room'
  | 'mixed'
  | 'unknown';

export type ProjectStatus = 'detected' | 'approved' | 'rejected' | 'archived' | 'deleted';

export interface Project {
  id: string;
  name: string;
  slug: string;
  description?: string | null;
  projectType: ProjectType;
  status: ProjectStatus;
  locationLabel?: string | null;
  latitude?: number | null;
  longitude?: number | null;
  startDate?: string | null;
  endDate?: string | null;
  customerName?: string | null;
  customerEmail?: string | null;
  customerPhone?: string | null;
  tags: string[];
  coverMediaId?: string | null;
  confidence: number;
  isPrivate: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface ProjectSummary {
  id: string;
  name: string;
  slug: string;
  status: ProjectStatus;
  projectType: ProjectType;
  confidence: number;
  mediaCount: number;
  firstDate?: string | null;
  lastDate?: string | null;
}

export const PROJECT_TYPE_LABELS: Record<ProjectType, string> = {
  sanitary_bathroom: 'Badkamerrenovatie',
  cv_boiler: 'CV-ketel installatie',
  heat_pump: 'Warmtepomp installatie',
  radiator: 'Radiator vervanging',
  ventilation: 'Ventilatiesysteem',
  airco: 'Airco-installatie',
  water_softener: 'Waterontharder',
  technical_room: 'Technische ruimte',
  mixed: 'Gemengd project',
  unknown: 'Onbekend',
};

export const PROJECT_STATUS_LABELS: Record<ProjectStatus, string> = {
  detected: 'Gedetecteerd',
  approved: 'Goedgekeurd',
  rejected: 'Afgewezen',
  archived: 'Gearchiveerd',
  deleted: 'Verwijderd',
};

export const PROJECT_TYPE_COLORS: Record<ProjectType, string> = {
  sanitary_bathroom: 'bg-blue-100 text-blue-800',
  cv_boiler: 'bg-orange-100 text-orange-800',
  heat_pump: 'bg-green-100 text-green-800',
  radiator: 'bg-red-100 text-red-800',
  ventilation: 'bg-cyan-100 text-cyan-800',
  airco: 'bg-indigo-100 text-indigo-800',
  water_softener: 'bg-teal-100 text-teal-800',
  technical_room: 'bg-gray-200 text-gray-800',
  mixed: 'bg-purple-100 text-purple-800',
  unknown: 'bg-gray-100 text-gray-700',
};
