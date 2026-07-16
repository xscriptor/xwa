import { Component, inject } from '@angular/core';
import { RouterOutlet } from '@angular/router';

@Component({
    selector: 'app-root',
    standalone: true,
    imports: [RouterOutlet],
    template: `
    <header>
      <h1><span>// </span>shinobi<span class="gold">.</span></h1>
      <span class="subtitle">web scraper — anti-blocking download system</span>
      <span class="status-line">
        <span class="dot" [style.background]="rustOk ? 'var(--success)' : 'var(--error)'"></span>
        rust
        <span class="dot" [style.background]="pythonOk ? 'var(--success)' : 'var(--text-disabled)'"></span>
        python
        <button class="theme-toggle" (click)="toggleTheme()">{{ isLight ? 'DARK' : 'LIGHT' }}</button>
      </span>
    </header>
    <div class="container">
      <router-outlet></router-outlet>
    </div>
    `,
})
export class AppComponent {
    rustOk = true;
    pythonOk = false;
    isLight = false;

    ngOnInit() {
        fetch('/api/health').then(r => this.rustOk = r.ok).catch(() => this.rustOk = false);
        fetch('http://localhost:9090/health').then(r => this.pythonOk = r.ok).catch(() => this.pythonOk = false);
        const saved = localStorage.getItem('shinobi-theme');
        if (saved === 'light') this.setTheme(true);
    }

    toggleTheme() {
        this.setTheme(!this.isLight);
    }

    private setTheme(light: boolean) {
        this.isLight = light;
        document.body.classList.toggle('theme-light', light);
        localStorage.setItem('shinobi-theme', light ? 'light' : 'dark');
    }
}
