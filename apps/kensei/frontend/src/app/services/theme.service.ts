import { Injectable, Inject, PLATFORM_ID } from '@angular/core';
import { isPlatformBrowser } from '@angular/common';

@Injectable({
  providedIn: 'root'
})
export class ThemeService {
  private readonly THEME_KEY = 'kensei-theme';

  constructor(@Inject(PLATFORM_ID) private platformId: object) {}

  initTheme(): void {
    if (isPlatformBrowser(this.platformId)) {
      const saved = localStorage.getItem(this.THEME_KEY);
      if (saved === 'light') {
        document.body.classList.remove('theme-dark');
        document.body.classList.add('theme-light');
      }
    }
  }

  toggleTheme(): void {
    const isDark = document.body.classList.contains('theme-dark');
    document.body.classList.add('theme-transitioning');
    
    if (isDark) {
      document.body.classList.replace('theme-dark', 'theme-light');
    } else {
      document.body.classList.replace('theme-light', 'theme-dark');
    }

    if (isPlatformBrowser(this.platformId)) {
      localStorage.setItem(this.THEME_KEY, isDark ? 'light' : 'dark');
    }

    setTimeout(() => {
      document.body.classList.remove('theme-transitioning');
    }, 300);
  }

  nextThemeLabel(): string {
    return document.body.classList.contains('theme-dark') ? 'LIGHT' : 'DARK';
  }

  nextThemeAriaLabel(): string {
    return `Switch to ${this.nextThemeLabel().toLowerCase()} theme`;
  }
}
