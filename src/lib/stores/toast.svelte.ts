// src/lib/stores/toast.svelte.ts

type ToastType = "success" | "error" | "warning" | "info";

interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration: number;
}

class ToastStore {
  toasts = $state<Toast[]>([]);

  show(message: string, type: ToastType = "info", duration = 4000): void {
    const id = crypto.randomUUID();
    this.toasts = [...this.toasts, { id, type, message, duration }];
    if (duration > 0) {
      setTimeout(() => this.dismiss(id), duration);
    }
  }

  success(message: string, duration = 4000): void {
    this.show(message, "success", duration);
  }

  error(message: string, duration = 6000): void {
    this.show(message, "error", duration);
  }

  warning(message: string, duration = 5000): void {
    this.show(message, "warning", duration);
  }

  info(message: string, duration = 4000): void {
    this.show(message, "info", duration);
  }

  dismiss(id: string): void {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  reset(): void {
    this.toasts = [];
  }
}

export const toastStore = new ToastStore();
