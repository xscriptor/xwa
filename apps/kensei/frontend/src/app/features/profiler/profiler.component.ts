import { Component, OnDestroy, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RouterModule } from '@angular/router';
import { HttpClient } from '@angular/common/http';

interface Technology {
  name: string;
  version: string | null;
  confidence: string;
  evidence: string;
}

interface ProfileReport {
  domain: string;
  profile_id: number;
  created_at: string;
  summary: {
    technologies_found: number;
    routes_discovered: number;
    guards_detected: number;
    js_dependencies_found: number;
    categories: string[];
  };
  technologies_by_category: Record<string, Technology[]>;
  outdated_technologies?: Technology[];
}

interface DiscoveredRoute {
  path: string;
  framework: string | null;
  route_type: string;
  module: string | null;
}

interface JsDependency {
  name: string;
  version: string | null;
  source: string | null;
  package_manager: string | null;
}

interface ProfileDetail {
  id: number;
  domain_target: string;
  status: string;
  created_at: string;
  technologies: Technology[];
  routes: DiscoveredRoute[];
  js_dependencies: JsDependency[];
}

@Component({
  selector: 'app-profiler',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterModule],
  templateUrl: './profiler.component.html',
  styleUrls: ['./profiler.component.scss']
})
export class ProfilerComponent implements OnDestroy {
  targetUrl = '';
  host = window.location.hostname;
  scanning = false;
  completed = false;
  error: string | null = null;
  profileId: number | null = null;
  report: ProfileReport | null = null;
  detail: ProfileDetail | null = null;

  currentPhase = '';
  techsFound = 0;
  routesFound = 0;
  guardsFound = 0;
  jsDepsFound = 0;
  currentTarget = '';

  phases = [
    { id: 1, label: 'SSL/TLS', done: false, active: false },
    { id: 2, label: 'SERVER FINGERPRINT', done: false, active: false },
    { id: 3, label: 'JS BUNDLE ANALYSIS', done: false, active: false },
    { id: 4, label: 'SPA ROUTE DISCOVERY', done: false, active: false },
  ];

  private ws: WebSocket | null = null;

  constructor(
    private http: HttpClient,
    private cdr: ChangeDetectorRef
  ) {}

  startScan(): void {
    const url = this.targetUrl.trim();
    if (!url || this.scanning) return;

    this.scanning = true;
    this.completed = false;
    this.error = null;
    this.profileId = null;
    this.report = null;
    this.detail = null;
    this.currentPhase = '';
    this.techsFound = 0;
    this.routesFound = 0;
    this.guardsFound = 0;
    this.jsDepsFound = 0;
    this.currentTarget = url;
    this.phases.forEach(p => { p.done = false; p.active = false; });
    this.cdr.detectChanges();

    const wsUrl = `ws://${this.host}:8000/api/profile/live?target=${encodeURIComponent(url)}&timeout=180`;
    this.ws = new WebSocket(wsUrl);

    this.ws.onmessage = (event) => {
      const msg = event.data;

      if (msg.startsWith('[!] CRITICAL ERROR:')) {
        this.error = msg.replace('[!] CRITICAL ERROR:', '').trim();
        this.scanning = false;
        this.cdr.detectChanges();
        return;
      }

      const metaMatch = msg.match(/\[PROFILE_META\] profile_id=(\d+)/);
      if (metaMatch) {
        this.profileId = parseInt(metaMatch[1], 10);
        this.cdr.detectChanges();
        return;
      }

      const phaseMatch = msg.match(/\[kensei\] phase (\d)\/4 — (.+)/);
      if (phaseMatch) {
        const num = parseInt(phaseMatch[1], 10);
        this.currentPhase = phaseMatch[2].trim();
        this.phases.forEach(p => {
          p.active = p.id === num;
          if (p.id < num) p.done = true;
        });
        this.cdr.detectChanges();
        return;
      }

      if (msg.includes('complete') && msg.includes('phase')) {
        const phaseDone = this.phases.find(p => p.active);
        if (phaseDone) { phaseDone.done = true; phaseDone.active = false; }
        this.cdr.detectChanges();
        return;
      }

      const techMatch = msg.match(/\[fingerprint\] .+? ([\w-]+)/);
      const cdnMatch = msg.match(/\[fingerprint\] CDN: (.+)/);
      if (techMatch || cdnMatch) {
        this.techsFound++;
        this.cdr.detectChanges();
        return;
      }

      const depMatch = msg.match(/\[js\] found (\d+) dependencies/);
      if (depMatch) {
        this.jsDepsFound += parseInt(depMatch[1], 10);
        this.cdr.detectChanges();
        return;
      }

      const routeMatch = msg.match(/\[spa\] found (\d+) routes/);
      if (routeMatch) {
        this.routesFound = parseInt(routeMatch[1], 10);
        this.cdr.detectChanges();
        return;
      }

      if (msg === '[done] profiling complete and saved to history') {
        this.completed = true;
        this.scanning = false;
        this.phases.forEach(p => { p.done = true; p.active = false; });
        this.ws?.close();
        this.fetchReport();
        this.fetchDetail();
        this.cdr.detectChanges();
      }
    };

    this.ws.onerror = () => {
      this.error = 'WebSocket connection failed. Is the backend on port 8000?';
      this.scanning = false;
      this.cdr.detectChanges();
    };

    this.ws.onclose = () => {
      if (!this.completed && !this.error) {
        this.error = 'Connection closed unexpectedly.';
      }
      this.scanning = false;
      this.cdr.detectChanges();
    };
  }

  fetchReport(): void {
    if (!this.profileId) return;
    this.http.get<ProfileReport>(`http://${this.host}:8000/api/profiles/${this.profileId}/report`)
      .subscribe({
        next: (r) => { this.report = r; this.cdr.detectChanges(); },
        error: () => { this.error = 'Failed to load profile report.'; this.cdr.detectChanges(); }
      });
  }

  fetchDetail(): void {
    if (!this.profileId) return;
    this.http.get<ProfileDetail>(`http://${this.host}:8000/api/profiles/${this.profileId}`)
      .subscribe({
        next: (d) => { this.detail = d; this.cdr.detectChanges(); },
        error: () => {}
      });
  }

  get guards(): DiscoveredRoute[] {
    return this.detail?.routes.filter(r => r.route_type === 'guard') || [];
  }

  get regularRoutes(): DiscoveredRoute[] {
    return this.detail?.routes.filter(r => r.route_type !== 'guard') || [];
  }

  ngOnDestroy(): void {
    this.ws?.close();
  }
}
