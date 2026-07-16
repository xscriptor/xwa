import { Component, OnInit, OnDestroy, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpClient } from '@angular/common/http';
import { Router, NavigationEnd } from '@angular/router';
import { filter, Subscription } from 'rxjs';

interface Profile {
  id: number;
  domain_target: string;
  status: string;
  created_at: string;
}

interface CompareResult {
  profiles: Array<{
    profile_id: number;
    domain: string;
    created_at: string;
    technology_count: number;
    technologies: string[];
  }>;
  changes: Array<{
    from_profile: number;
    to_profile: number;
    added: string[];
    removed: string[];
  }>;
}

@Component({
  selector: 'app-history',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './history.component.html',
  styleUrls: ['./history.component.scss']
})
export class HistoryComponent implements OnInit, OnDestroy {
  profiles: Profile[] = [];
  loading = true;
  error: string | null = null;
  host = window.location.hostname;

  selectedForCompare: Set<number> = new Set();
  compareResult: CompareResult | null = null;
  comparing = false;
  compareError: string | null = null;
  deletingAll = false;
  confirmDeleteId: number | 'all' | null = null;
  private routerSub: Subscription | null = null;

  constructor(
    private http: HttpClient,
    private cdr: ChangeDetectorRef,
    private router: Router
  ) {}

  ngOnInit(): void {
    this.loadProfiles();
    this.routerSub = this.router.events.pipe(
      filter(e => e instanceof NavigationEnd)
    ).subscribe(() => this.loadProfiles());
  }

  ngOnDestroy(): void {
    this.routerSub?.unsubscribe();
  }

  loadProfiles(): void {
    this.loading = true;
    this.http.get<Profile[]>(`http://${this.host}:8000/api/profiles`)
      .subscribe({
        next: (data) => { this.profiles = data; this.loading = false; this.cdr.detectChanges(); },
        error: () => { this.error = 'Failed to load profiles.'; this.loading = false; this.cdr.detectChanges(); }
      });
  }

  viewProfile(id: number): void {
    this.router.navigate(['/inventory'], { queryParams: { id } });
  }

  toggleCompare(id: number): void {
    if (this.selectedForCompare.has(id)) {
      this.selectedForCompare.delete(id);
    } else {
      this.selectedForCompare.add(id);
    }
    this.compareResult = null;
  }

  runCompare(): void {
    const ids = [...this.selectedForCompare].sort((a, b) => a - b);
    if (ids.length < 2) return;
    this.comparing = true;
    this.compareError = null;
    this.compareResult = null;
    this.http.get<CompareResult>(`http://${this.host}:8000/api/profiles/compare?ids=${ids.join(',')}`)
      .subscribe({
        next: (data) => { this.compareResult = data; this.comparing = false; this.cdr.detectChanges(); },
        error: () => { this.compareError = 'Compare failed.'; this.comparing = false; this.cdr.detectChanges(); }
      });
  }

  requestDelete(id: number | 'all'): void {
    this.confirmDeleteId = id;
  }

  cancelDelete(): void {
    this.confirmDeleteId = null;
  }

  confirmDelete(): void {
    if (this.confirmDeleteId === 'all') {
      this.deletingAll = true;
      this.confirmDeleteId = null;
      this.http.delete(`http://${this.host}:8000/api/profiles`).subscribe({
        next: () => { this.deletingAll = false; this.profiles = []; this.selectedForCompare.clear(); this.compareResult = null; this.cdr.detectChanges(); },
        error: () => { this.deletingAll = false; this.error = 'Failed to delete all profiles.'; this.cdr.detectChanges(); }
      });
    } else if (this.confirmDeleteId !== null) {
      const id = this.confirmDeleteId;
      this.confirmDeleteId = null;
      this.http.delete(`http://${this.host}:8000/api/profiles/${id}`)
        .subscribe({ next: () => { this.selectedForCompare.delete(id); this.loadProfiles(); } });
    }
  }

  exportJson(id: number): void {
    window.open(`http://${this.host}:8000/api/profiles/${id}/export/json`, '_blank');
  }

  statusClass(status: string): string {
    return status.toLowerCase();
  }
}
