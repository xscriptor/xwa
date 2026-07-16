import { Component, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { HttpClient } from '@angular/common/http';
import { TranslatePipe } from '../../pipes/translate.pipe';

interface ExportSummary {
  scan_count: number;
  finding_count: number;
  link_count: number;
}

@Component({
  selector: 'app-export-database',
  standalone: true,
  imports: [CommonModule, FormsModule, TranslatePipe],
  templateUrl: './export-database.component.html',
  styleUrls: ['./export-database.component.scss'],
})
export class ExportDatabaseComponent {
  exportMode: 'direct' | 'encrypted' = 'direct';
  encryptPassword = '';
  isExporting = false;
  exportDone = false;
  errorMessage = '';
  summary: ExportSummary | null = null;
  lastExportFilename = '';

  private baseUrl = `http://${window.location.hostname}:8000`;

  constructor(private http: HttpClient, private cdr: ChangeDetectorRef) {}

  selectMode(mode: 'direct' | 'encrypted'): void {
    this.exportMode = mode;
    this.exportDone = false;
    this.errorMessage = '';
    this.summary = null;
  }

  async exportDirect(): Promise<void> {
    this.isExporting = true;
    this.exportDone = false;
    this.errorMessage = '';

    try {
      const resp = await this.http
        .get(`${this.baseUrl}/api/database/export/raw`, {
          responseType: 'blob',
          observe: 'response',
        })
        .toPromise();

      if (!resp || !resp.body) {
        this.errorMessage = '[ERROR] Empty response from server';
        this.isExporting = false;
        this.cdr.detectChanges();
        return;
      }

      await this.processJsonResponse(resp.body);
      const filename = this.generateFilename('json');
      this.downloadBlob(resp.body, filename);
      this.lastExportFilename = filename;
      this.exportDone = true;
    } catch (err: any) {
      this.errorMessage = `[ERROR] ${err?.message || 'Export failed'}`;
    } finally {
      this.isExporting = false;
      this.cdr.detectChanges();
    }
  }

  async exportEncrypted(): Promise<void> {
    if (!this.encryptPassword || this.encryptPassword.length < 4) {
      this.errorMessage = '[ERROR] Password must be at least 4 characters';
      this.cdr.detectChanges();
      return;
    }

    this.isExporting = true;
    this.exportDone = false;
    this.errorMessage = '';

    try {
      const resp = await this.http
        .post(
          `${this.baseUrl}/api/database/export/encrypted`,
          { password: this.encryptPassword },
          {
            responseType: 'blob',
            observe: 'response',
          }
        )
        .toPromise();

      if (!resp || !resp.body) {
        this.errorMessage = '[ERROR] Empty response from server';
        this.isExporting = false;
        this.cdr.detectChanges();
        return;
      }

      if (resp.body.type === 'application/json') {
        const text = await resp.body.text();
        const errData = JSON.parse(text);
        this.errorMessage = `[ERROR] ${errData.detail || 'Export failed'}`;
        this.isExporting = false;
        this.cdr.detectChanges();
        return;
      }

      const filename = this.generateFilename('bin.enc');
      this.downloadBlob(resp.body, filename);
      this.lastExportFilename = filename;
      this.exportDone = true;
    } catch (err: any) {
      this.errorMessage = `[ERROR] ${err?.message || 'Encrypted export failed'}`;
    } finally {
      this.isExporting = false;
      this.cdr.detectChanges();
    }
  }

  private async processJsonResponse(blob: Blob): Promise<void> {
    try {
      const text = await blob.text();
      const data = JSON.parse(text);
      this.summary = {
        scan_count: data.export_metadata?.scan_count ?? 0,
        finding_count: data.export_metadata?.finding_count ?? 0,
        link_count: data.export_metadata?.link_count ?? 0,
      };
    } catch {
      this.summary = null;
    }
  }

  private generateFilename(ext: string): string {
    const date = new Date().toISOString().split('T')[0];
    return `samurai-database-export-${date}.${ext}`;
  }

  private downloadBlob(blob: Blob, filename: string): void {
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = filename;
    document.body.appendChild(anchor);
    anchor.click();
    document.body.removeChild(anchor);
    URL.revokeObjectURL(url);
  }
}
