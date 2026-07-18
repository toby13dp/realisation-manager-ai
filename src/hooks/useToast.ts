/// Toast hook.

import { useToastStore } from '@/store/toastStore';

export function useToast() {
  const toasts = useToastStore((s) => s.toasts);
  const dismiss = useToastStore((s) => s.dismiss);
  const push = useToastStore((s) => s.push);
  return { toasts, dismiss, push };
}
