import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpClient } from '@angular/common/http';

interface ProfileSummary {
  id: number;
  domain_target: string;
}

interface DiscoveredRoute {
  path: string;
  framework: string | null;
  route_type: string;
  module: string | null;
}

interface ProfileDetail {
  id: number;
  domain_target: string;
  created_at: string;
  routes: DiscoveredRoute[];
}

@Component({
  selector: 'app-spa-routes',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './spa-routes.component.html',
  styleUrls: ['./spa-routes.component.scss']
})
export class SpaRoutesComponent implements OnInit {
  profiles: ProfileSummary[] = [];
  profile: ProfileDetail | null = null;
  loading = false;
  error: string | null = null;
  host = window.location.hostname;

  constructor(
    private http: HttpClient,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.http.get<ProfileSummary[]>(`http://${this.host}:8000/api/profiles`)
      .subscribe({ next: (data) => { this.profiles = data; this.cdr.detectChanges(); } });
  }

  loadProfile(id: number): void {
    this.loading = true;
    this.error = null;
    this.profile = null;
    this.http.get<ProfileDetail>(`http://${this.host}:8000/api/profiles/${id}`)
      .subscribe({
        next: (data) => { this.profile = data; this.loading = false; this.cdr.detectChanges(); },
        error: () => { this.error = 'Failed to load.'; this.loading = false; this.cdr.detectChanges(); }
      });
  }

  get guards(): DiscoveredRoute[] {
    return this.profile?.routes.filter(r => r.route_type === 'guard') || [];
  }

  get regularRoutes(): DiscoveredRoute[] {
    return this.profile?.routes.filter(r => r.route_type !== 'guard') || [];
  }
}
