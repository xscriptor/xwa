import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpClient } from '@angular/common/http';

interface ProfileSummary {
  id: number;
  domain_target: string;
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
  created_at: string;
  js_dependencies: JsDependency[];
}

interface VersionDb {
  [lib: string]: { latest: string; all: string[] };
}

@Component({
  selector: 'app-js-analysis',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './js-analysis.component.html',
  styleUrls: ['./js-analysis.component.scss']
})
export class JsAnalysisComponent implements OnInit {
  profiles: ProfileSummary[] = [];
  profile: ProfileDetail | null = null;
  loading = false;
  error: string | null = null;
  host = window.location.hostname;

  versionDb: VersionDb | null = null;
  versionDbLoading = false;
  showVersionDb = false;

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

  toggleVersionDb(): void {
    this.showVersionDb = !this.showVersionDb;
    if (this.showVersionDb && !this.versionDb) {
      this.versionDbLoading = true;
      this.http.get<VersionDb>(`http://${this.host}:8000/api/version-db`)
        .subscribe({
          next: (data) => { this.versionDb = data; this.versionDbLoading = false; this.cdr.detectChanges(); },
          error: () => { this.versionDbLoading = false; this.cdr.detectChanges(); }
        });
    }
  }

  dbKeys(): string[] {
    return this.versionDb ? Object.keys(this.versionDb).sort() : [];
  }
}
