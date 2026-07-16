import { Injectable } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class ConfirmService {
  private resolver: ((value: boolean) => void) | null = null;

  active = false;
  message = '';
  title = '';

  ask(title: string, message: string): Promise<boolean> {
    this.title = title;
    this.message = message;
    this.active = true;
    return new Promise(resolve => {
      this.resolver = resolve;
    });
  }

  confirm() {
    this.active = false;
    this.resolver?.(true);
    this.resolver = null;
  }

  cancel() {
    this.active = false;
    this.resolver?.(false);
    this.resolver = null;
  }
}
