import { Injectable } from '@angular/core';

export interface Toast {
  id: number;
  message: string;
  type: 'ok' | 'error' | 'warn';
}

@Injectable({ providedIn: 'root' })
export class ToastService {
  private id = 0;
  toasts: Toast[] = [];

  show(message: string, type: Toast['type'] = 'ok', duration = 4000) {
    const id = ++this.id;
    this.toasts.push({ id, message, type });
    setTimeout(() => this.dismiss(id), duration);
  }

  ok(message: string) { this.show(message, 'ok'); }
  error(message: string) { this.show(message, 'error', 8000); }
  warn(message: string) { this.show(message, 'warn', 6000); }

  dismiss(id: number) {
    this.toasts = this.toasts.filter(t => t.id !== id);
  }
}
