import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpClient } from '@angular/common/http';
import { RouterModule } from '@angular/router';
import { TranslatePipe } from '../../pipes/translate.pipe';

interface Finding {
  category: string;
  check: string;
  severity: string;
  title: string;
}

interface AuditRecord {
  id: string;
  url: string;
  status: string;
  findings?: Finding[];
  created_at: string;
}

@Component({
  selector: 'app-history',
  standalone: true,
  imports: [CommonModule, RouterModule, TranslatePipe],
  templateUrl: './history.component.html',
  styleUrls: ['./history.component.scss'],
})
export class HistoryComponent implements OnInit {
  audits: AuditRecord[] = [];
  isLoading = false;
  clearing = false;
  errorMessage = '';

  compareMode = false;
  selectedIds: Set<string> = new Set();
  comparisonResult: { a: AuditRecord | null; b: AuditRecord | null; diff: string[] } | null = null;

  constructor(private http: HttpClient) {}

  ngOnInit(): void {
    this.loadHistory();
  }

  statusClass(status: string): string {
    switch (status) {
      case 'COMPLETED': return 'text-success';
      case 'RUNNING': return 'text-warning';
      case 'ERROR': return 'text-error';
      default: return 'text-secondary';
    }
  }

  loadHistory(): void {
    this.isLoading = true;
    this.errorMessage = '';
    this.comparisonResult = null;

    this.http.get<AuditRecord[]>('/api/audits').subscribe({
      next: (data) => {
        this.audits = data.sort((a, b) => b.created_at.localeCompare(a.created_at));
        this.isLoading = false;
      },
      error: (err) => {
        this.errorMessage = `Failed to load history: ${err.message || err.statusText || 'Connection refused'}`;
        this.isLoading = false;
      },
    });
  }

  clearAll(): void {
    if (!confirm('Delete all audit history? This action cannot be undone.')) {
      return;
    }
    this.clearing = true;
    this.http.delete('/api/audits/clear').subscribe({
      next: () => {
        this.audits = [];
        this.clearing = false;
        this.comparisonResult = null;
        this.selectedIds.clear();
      },
      error: (err) => {
        this.errorMessage = `Failed to clear history: ${err.message || 'Unknown error'}`;
        this.clearing = false;
      },
    });
  }

  toggleCompare(): void {
    this.compareMode = !this.compareMode;
    this.selectedIds.clear();
    this.comparisonResult = null;
  }

  toggleSelection(id: string): void {
    if (this.selectedIds.has(id)) {
      this.selectedIds.delete(id);
      this.comparisonResult = null;
      return;
    }
    if (this.selectedIds.size >= 2) {
      return;
    }
    this.selectedIds.add(id);
    if (this.selectedIds.size === 2) {
      this.runComparison();
    }
  }

  private runComparison(): void {
    const ids = [...this.selectedIds];
    const a = this.audits.find(a => a.id === ids[0]) || null;
    const b = this.audits.find(a => a.id === ids[1]) || null;
    if (!a || !b) { this.comparisonResult = null; return; }

    const diff: string[] = [];
    const af = a.findings || [];
    const bf = b.findings || [];

    // Findings in A not in B
    for (const fa of af) {
      const match = bf.find(fb => fb.check === fa.check && fb.category === fa.category);
      if (!match) {
        diff.push(`[-] ${fa.category}/${fa.check} — present in A, fixed in B`);
      } else if (match.severity !== fa.severity) {
        diff.push(`[~] ${fa.category}/${fa.check} — severity changed: ${fa.severity} → ${match.severity}`);
      }
    }
    // Findings in B not in A
    for (const fb of bf) {
      const match = af.find(fa => fa.check === fb.check && fa.category === fb.category);
      if (!match) {
        diff.push(`[+] ${fb.category}/${fb.check} — new in B`);
      }
    }

    this.comparisonResult = { a, b, diff };
  }

  compareCategoryCount(a: AuditRecord | null, cat: string): number {
    return (a?.findings || []).filter(f => f.category === cat).length;
  }

  compareSeverityCount(a: AuditRecord | null, sev: string): number {
    return (a?.findings || []).filter(f => f.severity === sev).length;
  }

  get scoreHistory(): { date: string; url: string; total: number; errors: number; warnings: number }[] {
    return this.audits
      .filter(a => a.status === 'COMPLETED' && a.findings)
      .map(a => ({
        date: a.created_at.slice(0, 10),
        url: a.url,
        total: a.findings?.length || 0,
        errors: (a.findings || []).filter(f => f.severity === 'Error').length,
        warnings: (a.findings || []).filter(f => f.severity === 'Warning').length,
      }));
  }

  get maxScore(): number {
    const h = this.scoreHistory;
    if (h.length === 0) return 1;
    return Math.max(...h.map(s => s.total + s.errors + s.warnings), 1);
  }

  get chartWidth(): number {
    const len = this.scoreHistory.length;
    return Math.max(200, 40 + len * 60);
  }

  get stepX(): number {
    const len = this.scoreHistory.length;
    return len > 1 ? (this.chartWidth - 50) / (len - 1) : 0;
  }

  trendPoints(field: 'errors' | 'warnings' | 'total'): string {
    const h = this.scoreHistory;
    const max = this.maxScore || 1;
    return h.map((s, i) => {
      const x = 40 + i * this.stepX;
      const y = 130 - ((s[field] / max) * 100);
      return `${x},${y}`;
    }).join(' ');
  }
}
