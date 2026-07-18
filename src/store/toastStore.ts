/// Toast notifications store.

import { create } from 'zustand';

export interface Toast {
  id: string;
  type: 'success' | 'error' | 'info' | 'warning';
  message: string;
  timeout?: number;
}

interface ToastStoreState {
  toasts: Toast[];
  push: (toast: Omit<Toast, 'id'>) => void;
  dismiss: (id: string) => void;
}

export const useToastStore = create<ToastStoreState>((set) => ({
  toasts: [],
  push: (toast) => {
    const id = Math.random().toString(36).slice(2, 9);
    set((s) => ({ toasts: [...s.toasts, { ...toast, id }] }));
    const timeout = toast.timeout ?? 4000;
    if (timeout > 0) {
      setTimeout(() => {
        set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }));
      }, timeout);
    }
  },
  dismiss: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
}));

export const toast = {
  success: (message: string) => useToastStore.getState().push({ type: 'success', message }),
  error: (message: string) => useToastStore.getState().push({ type: 'error', message }),
  info: (message: string) => useToastStore.getState().push({ type: 'info', message }),
  warning: (message: string) => useToastStore.getState().push({ type: 'warning', message }),
};
