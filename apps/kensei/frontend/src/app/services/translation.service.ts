import { Injectable } from '@angular/core';

@Injectable({
  providedIn: 'root'
})
export class TranslationService {
  private currentLang: 'en' | 'es' = 'en';

  initLang(): void {
    this.currentLang = 'en';
  }

  toggleLang(): void {
    this.currentLang = this.currentLang === 'en' ? 'es' : 'en';
  }

  nextLangLabel(): string {
    return this.currentLang === 'en' ? 'ES' : 'EN';
  }

  nextLangAria(): string {
    return `Switch language to ${this.nextLangLabel()}`;
  }
}
