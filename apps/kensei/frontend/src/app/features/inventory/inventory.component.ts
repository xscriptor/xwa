import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpClient } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';
import jsPDF from 'jspdf';
import autoTable from 'jspdf-autotable';
import { gzip } from 'pako';

interface Technology {
  category: string;
  name: string;
  version: string | null;
  confidence: string;
  evidence: string;
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
  selector: 'app-inventory',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './inventory.component.html',
  styleUrls: ['./inventory.component.scss']
})
export class InventoryComponent implements OnInit {
  profile: ProfileDetail | null = null;
  profiles: { id: number; domain_target: string }[] = [];
  loading = false;
  error: string | null = null;
  selectedId: number | null = null;
  host = window.location.hostname;
  private loadingFromParams = false;

  constructor(
    private http: HttpClient,
    private cdr: ChangeDetectorRef,
    private route: ActivatedRoute,
    private router: Router
  ) {}

  ngOnInit(): void {
    this.http.get<{ id: number; domain_target: string }[]>(`http://${this.host}:8000/api/profiles`)
      .subscribe({ next: (data) => { this.profiles = data; this.cdr.detectChanges(); } });

    this.route.queryParams.subscribe(params => {
      if (params['id'] && !this.loadingFromParams) {
        this.selectedId = parseInt(params['id'], 10);
        this.loadProfile(this.selectedId);
      }
    });
  }

  loadProfile(id: number): void {
    this.loading = true;
    this.error = null;
    this.selectedId = id;
    this.profile = null;
    this.loadingFromParams = true;
    this.router.navigate([], { queryParams: { id }, replaceUrl: true });
    this.cdr.detectChanges();

    this.http.get<ProfileDetail>(`http://${this.host}:8000/api/profiles/${id}`)
      .subscribe({
        next: (data) => {
          this.profile = data;
          this.loading = false;
          this.loadingFromParams = false;
          this.cdr.detectChanges();
        },
        error: () => {
          this.error = 'Failed to load profile details.';
          this.loading = false;
          this.loadingFromParams = false;
          this.cdr.detectChanges();
        }
      });
  }

  groupedTechs(): Record<string, Technology[]> {
    if (!this.profile) return {};
    const groups: Record<string, Technology[]> = {};
    for (const t of this.profile.technologies) {
      (groups[t.category] ??= []).push(t);
    }
    return groups;
  }

  exportJson(): void {
    if (!this.profile) return;
    const payload = JSON.stringify(this.profile, null, 2);
    this.download(`${this.profile.domain_target}-profile-${this.profile.id}.json`, payload, 'application/json');
  }

  exportCsv(): void {
    if (!this.profile) return;
    const rows: string[] = ['category,name,version,confidence,evidence'];
    for (const t of this.profile.technologies) {
      rows.push([t.category, t.name, t.version || '', t.confidence, (t.evidence || '').replace(/"/g, '""')].map(v => `"${v}"`).join(','));
    }
    this.download(`${this.profile.domain_target}-profile-${this.profile.id}.csv`, rows.join('\n'), 'text/csv;charset=utf-8');
  }

  exportPdf(): void {
    if (!this.profile) return;
    const doc = new jsPDF({ orientation: 'landscape', unit: 'pt', format: 'a4' });
    doc.setFontSize(14);
    doc.text('Kensei Profile Report', 40, 36);
    doc.setFontSize(10);
    doc.text(`Target: ${this.profile.domain_target}`, 40, 54);
    doc.text(`Status: ${this.profile.status} | ID: ${this.profile.id}`, 40, 68);
    doc.text(`Created: ${this.profile.created_at}`, 40, 82);
    autoTable(doc, {
      startY: 96,
      head: [['Category', 'Name', 'Version', 'Confidence', 'Evidence']],
      body: this.profile.technologies.map(t => [t.category, t.name, t.version || '', t.confidence, t.evidence || '']),
      styles: { fontSize: 8, cellPadding: 3, overflow: 'linebreak' },
      headStyles: { fillColor: [35, 35, 35] },
    });
    doc.save(`${this.profile.domain_target}-profile-${this.profile.id}.pdf`);
  }

  exportBinary(): void {
    if (!this.profile) return;
    const bytes = gzip(JSON.stringify(this.profile));
    const blob = new Blob([bytes], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${this.profile.domain_target}-profile-${this.profile.id}.bin`;
    a.click();
    URL.revokeObjectURL(url);
  }

  private download(name: string, content: string, mime: string): void {
    const blob = new Blob([content], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = name;
    a.click();
    URL.revokeObjectURL(url);
  }
}
