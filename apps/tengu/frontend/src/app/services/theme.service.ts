import { Injectable, signal } from '@angular/core';

@Injectable({
  providedIn: 'root',
})
export class ThemeService {
  private readonly storageKey = 'tengu-theme';
  readonly isDark = signal(true);

  constructor() {
    const saved = localStorage.getItem(this.storageKey);
    if (saved === 'light') {
      this.isDark.set(false);
      document.body.classList.add('theme-light');
    }
  }

  toggle(): void {
    const next = !this.isDark();
    this.isDark.set(next);
    document.body.classList.toggle('theme-light', !next);
    localStorage.setItem(this.storageKey, next ? 'dark' : 'light');
  }
}
