import { Component, OnInit, OnDestroy, inject, HostListener } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ApiService } from '../services/api.service';
import { ScrapeConfig, JobInfo, FileInfo, DeepConfig, DeepResult } from '../models/models';

@Component({
    selector: 'app-dashboard',
    standalone: true,
    imports: [CommonModule, FormsModule],
    templateUrl: './dashboard.component.html',
})
export class DashboardComponent implements OnInit, OnDestroy {
    private api = inject(ApiService);
    private statusTimeout: any = null;
    statusMsg = '';
    statusType: 'ok' | 'error' | 'warn' | '' = '';

    mode: 'fast' | 'deep' = 'fast';
    deepSubMode: 'single' | 'crawl' | 'batch' | 'pycrawl' = 'single';

    config: ScrapeConfig = {
        url: '', depth: 2, max_pages: 100, delay_ms: 1000, concurrency: 3,
        same_domain_only: true, download_assets: true, user_agent_rotation: true,
        javascript_rendering: false, respect_robots_txt: true, deduplicate: true,
        take_screenshots: false, extract_emails: false, rewrite_urls: true,
        generate_index: false, export_warc: false, rate_limit: 0,
    };

    deepConfig: DeepConfig = { url: '', extract_structured: true, nlp_enabled: false, custom_selectors: [] };
    deepCrawlConfig = { url: '', depth: 2, max_pages: 50, extract_structured: true, nlp_enabled: false };
    pyCrawlConfig = { url: '', depth: 3, max_pages: 100, extract_structured: true, nlp_enabled: true };
    pyCrawlResult: any = null;
    pyCrawlJobId = '';
    pyCrawlLog: string[] = [];
    pyCrawlPages = 0;
    pyCrawlFiles = 0;
    pyCrawlStatus = '';
    pyCrawlProgress = 0;
    pyCrawlResults: any[] = [];
    pyCrawlSelectedResult: any = null;
    pyCrawlTimer: any = null;

    batchUrls = '';
    customSelectorInput = '';

    jobs: JobInfo[] = [];
    files: FileInfo[] = [];
    deepResults: DeepResult[] = [];
    selectedDeepResult: DeepResult | null = null;
    schedules: any[] = [];
    activeTab: 'jobs' | 'files' | 'deep' | 'schedules' = 'jobs';

    activeJob: JobInfo | null = null;
    progressPct = 0;
    starting = false;
    deepStarting = false;
    loadingJobs = false;
    loadingFiles = false;
    loadingDeep = false;
    loadingSchedules = false;
    deleting = false;
    pageJobs = 0; totalJobs = 0; pageFiles = 0; totalFiles = 0; pageDeep = 0; totalDeep = 0;
    pageSize = 25;
    diskStats: any = null;

    newScheduleUrl = '';
    newScheduleInterval = 60;

    searchQuery = '';
    previewFile: FileInfo | null = null;
    previewContent: string | null = null;
    previewLoading = false;

    showKeys = false;
    private stream: EventSource | null = null;

    cancelPyCrawl() {
        if (!this.pyCrawlJobId) return;
        this.api.cancelCrawl(this.pyCrawlJobId).then(() => {
            clearInterval(this.pyCrawlTimer);
            this.pyCrawlStatus = 'cancelled';
            this.setStatus('Crawl cancelled');
        }).catch(() => this.setStatus('Cancel failed', 'error'));
    }

    @HostListener('document:keydown', ['$event'])
    handleKeydown(e: KeyboardEvent) {
        if (e.key === '?' && !e.ctrlKey && !e.metaKey) { this.showKeys = !this.showKeys; return; }
        if (e.key === 'Escape') { this.previewFile = null; this.previewContent = null; return; }
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
            if (this.mode === 'fast') this.startScrape();
            else if (this.deepSubMode === 'single') this.startDeepSingle();
            else if (this.deepSubMode === 'crawl') this.startDeepCrawl();
            else if (this.deepSubMode === 'batch') this.startDeepBatch();
            else if (this.deepSubMode === 'pycrawl') this.startPyCrawl();
        }
    }

    ngOnInit() {
        this.loadJobs(); this.loadFiles(); this.loadDeepResults(); this.loadSchedules(); this.loadStats();
        setInterval(() => this.loadJobs(), 3000);
    }

    prevPage(tab: string) {
        if (tab === 'jobs') { this.pageJobs = Math.max(0, this.pageJobs - this.pageSize); this.loadJobs(); }
        if (tab === 'files') { this.pageFiles = Math.max(0, this.pageFiles - this.pageSize); this.loadFiles(); }
        if (tab === 'deep') { this.pageDeep = Math.max(0, this.pageDeep - this.pageSize); this.loadDeepResults(); }
    }
    nextPage(tab: string) {
        if (tab === 'jobs') { if (this.pageJobs + this.pageSize < this.totalJobs) { this.pageJobs += this.pageSize; this.loadJobs(); } }
        if (tab === 'files') { if (this.pageFiles + this.pageSize < this.totalFiles) { this.pageFiles += this.pageSize; this.loadFiles(); } }
        if (tab === 'deep') { if (this.pageDeep + this.pageSize < this.totalDeep) { this.pageDeep += this.pageSize; this.loadDeepResults(); } }
    }
    getPageInfo(tab: string): string {
        const total = tab === 'jobs' ? this.totalJobs : tab === 'files' ? this.totalFiles : this.totalDeep;
        const offset = tab === 'jobs' ? this.pageJobs : tab === 'files' ? this.pageFiles : this.pageDeep;
        if (!total) return '';
        return `${offset + 1}-${Math.min(offset + this.pageSize, total)} / ${total}`;
    }

    ngOnDestroy() { this.stream?.close(); if (this.pyCrawlTimer) clearInterval(this.pyCrawlTimer); }

    setStatus(msg: string, type: 'ok' | 'error' | 'warn' = 'ok') {
        this.statusMsg = msg; this.statusType = type;
        if (this.statusTimeout) clearTimeout(this.statusTimeout);
        this.statusTimeout = setTimeout(() => { this.statusMsg = ''; this.statusType = ''; }, 5000);
    }

    get stats() {
        const j = this.jobs;
        return {
            total: j.length,
            running: j.filter(x => x.status === 'running').length,
            pages: j.reduce((a, b) => a + b.pages_scraped, 0),
            files: j.reduce((a, b) => a + b.files_downloaded, 0),
            deep: this.deepResults.length,
        };
    }

    get filteredJobs() {
        if (!this.searchQuery) return this.jobs;
        const q = this.searchQuery.toLowerCase();
        return this.jobs.filter(j => j.url.toLowerCase().includes(q) || j.id.includes(q) || j.emails.some(e => e.includes(q)));
    }

    get filteredFiles() {
        if (!this.searchQuery) return this.files;
        const q = this.searchQuery.toLowerCase();
        return this.files.filter(f => f.path.toLowerCase().includes(q) || f.name.toLowerCase().includes(q));
    }

    get filteredDeep() {
        if (!this.searchQuery) return this.deepResults;
        const q = this.searchQuery.toLowerCase();
        return this.deepResults.filter(r => r.url.toLowerCase().includes(q) || r.extracted.emails.some(e => e.includes(q)));
    }

    startScrape() {
        if (!this.config.url.trim()) return;
        this.starting = true; this.config.deep_mode = false;
        this.api.startScrape(this.config).then(job => {
            this.activeJob = job; this.starting = false; this.streamJob(job.id); this.loadJobs();
            this.setStatus('Job started');
        }).catch(e => { this.setStatus('Scrape failed: ' + (e.message || ''), 'error'); this.starting = false; });
    }

    startDeepSingle() {
        if (!this.deepConfig.url.trim()) return;
        this.deepStarting = true;
        const body: any = { ...this.deepConfig };
        if (this.customSelectorInput.trim()) body.custom_selectors = this.customSelectorInput.split(',').map((s: string) => s.trim()).filter(Boolean);
        else delete body.custom_selectors;
        this.api.startDeepScrape(body).then(r => {
            this.deepStarting = false; this.deepResults.unshift(r); this.selectedDeepResult = r; this.activeTab = 'deep';
            this.setStatus('Extraction complete');
        }).catch(e => { this.setStatus('Deep research: ' + (e.message || ''), 'error'); this.deepStarting = false; });
    }

    startDeepCrawl() {
        if (!this.deepCrawlConfig.url.trim()) return;
        this.starting = true;
        const body: ScrapeConfig = { url: this.deepCrawlConfig.url, depth: this.deepCrawlConfig.depth, max_pages: this.deepCrawlConfig.max_pages, deep_mode: true, extract_structured: this.deepCrawlConfig.extract_structured, nlp_enabled: this.deepCrawlConfig.nlp_enabled, download_assets: false, same_domain_only: true, respect_robots_txt: true, delay_ms: 1000, concurrency: 3, user_agent_rotation: true };
        this.api.startScrape(body).then(job => {
            this.activeJob = job; this.starting = false; this.streamJob(job.id); this.loadJobs();
            this.setStatus('Deep crawl started');
        }).catch(e => { this.setStatus('Deep crawl: ' + (e.message || ''), 'error'); this.starting = false; });
    }

    startDeepBatch() {
        const urls = this.batchUrls.split('\n').map(u => u.trim()).filter(Boolean);
        if (!urls.length) { this.setStatus('Enter at least one URL', 'warn'); return; }
        this.deepStarting = true;
        this.api.startDeepBatch({ urls, extract_structured: true, nlp_enabled: false }).then(results => {
            this.deepStarting = false;
            for (const r of results) this.deepResults.unshift(r);
            if (results.length) this.selectedDeepResult = results[0]; this.activeTab = 'deep';
            this.setStatus(`Extracted ${results.length} URLs`);
        }).catch(e => { this.setStatus('Batch: ' + (e.message || ''), 'error'); this.deepStarting = false; });
    }

    startPyCrawl() {
        if (!this.pyCrawlConfig.url.trim()) return;
        this.deepStarting = true;
        this.pyCrawlResult = null;
        this.pyCrawlJobId = '';
        this.pyCrawlLog = [];
        this.pyCrawlPages = 0;
        this.pyCrawlFiles = 0;
        this.pyCrawlStatus = 'starting';
        this.pyCrawlProgress = 0;
        this.pyCrawlResults = [];
        this.pyCrawlSelectedResult = null;

        this.api.startPythonCrawl({
            url: this.pyCrawlConfig.url,
            depth: this.pyCrawlConfig.depth,
            max_pages: this.pyCrawlConfig.max_pages,
            extract_structured: this.pyCrawlConfig.extract_structured,
            nlp_enabled: this.pyCrawlConfig.nlp_enabled,
        }).then(r => {
            this.pyCrawlJobId = r.job_id;
            this.deepStarting = false;
            this.pyCrawlStatus = 'running';
            this.setStatus('Crawl started');
            this._pollPyCrawl();
        }).catch(e => {
            this.setStatus('Python crawl: ' + (e.message || ''), 'error');
            this.deepStarting = false;
        });
    }

    private _pollPyCrawl() {
        if (!this.pyCrawlJobId) return;
        this.pyCrawlTimer = setInterval(async () => {
            try {
                const data = await this.api.getCrawlStatus(this.pyCrawlJobId);
                this.pyCrawlStatus = data.status;
                this.pyCrawlPages = data.pages || 0;
                this.pyCrawlFiles = data.files || 0;
                this.pyCrawlProgress = data.progress_pct || 0;
                if (data.log) this.pyCrawlLog = data.log;

                if (['completed', 'failed', 'cancelled'].includes(data.status)) {
                    clearInterval(this.pyCrawlTimer);
                    if (data.status === 'completed') {
                        this._fetchPyCrawlResults();
                    } else {
                        this.setStatus(`Crawl ${data.status}`, 'error');
                    }
                }
            } catch { clearInterval(this.pyCrawlTimer); }
        }, 1000);
    }

    private async _fetchPyCrawlResults() {
        try {
            const data = await this.api.getCrawlResults(this.pyCrawlJobId);
            this.pyCrawlResult = data;
            this.pyCrawlResults = data.results || [];
            if (this.pyCrawlResults.length) this.pyCrawlSelectedResult = this.pyCrawlResults[0];
            this.setStatus(`Crawled ${data.pages || 0} pages, ${data.files || 0} files`);
        } catch { this.setStatus('Failed to fetch results', 'error'); }
    }

    private streamJob(id: string) {
        this.stream?.close();
        this.stream = this.api.jobStream(id);
        this.stream.onmessage = (e) => {
            const data: JobInfo = JSON.parse(e.data);
            this.activeJob = data;
            this.progressPct = data.total_pages > 0 ? Math.min(100, Math.round((data.pages_scraped / data.total_pages) * 100)) : 0;
            if (['completed', 'failed', 'cancelled'].includes(data.status)) {
                this.stream?.close(); this.stream = null; this.loadJobs(); this.loadFiles(); this.loadDeepResults();
            }
        };
    }

    cancelJob(id: string) { this.api.cancelJob(id).then(() => { if (this.activeJob?.id === id) { this.activeJob = null; this.stream?.close(); this.stream = null; } this.loadJobs(); }); }

    loadJobs() {
        this.loadingJobs = true;
        this.api.listJobs(this.pageJobs, this.pageSize).then(r => { this.jobs = r.items; this.totalJobs = r.total; }).catch(() => this.setStatus('Failed to load jobs', 'error')).finally(() => this.loadingJobs = false);
    }
    loadFiles() {
        this.loadingFiles = true;
        this.api.listFiles('', this.pageFiles, this.pageSize).then(r => { this.files = r.items; this.totalFiles = r.total; }).catch(() => this.setStatus('Failed to load files', 'error')).finally(() => this.loadingFiles = false);
    }
    loadDeepResults() {
        this.loadingDeep = true;
        this.api.listDeepResults(this.pageDeep, this.pageSize).then(r => { this.deepResults = r.items; this.totalDeep = r.total; }).catch(() => {}).finally(() => this.loadingDeep = false);
    }
    loadSchedules() { this.loadingSchedules = true; this.api.listSchedules().then(s => this.schedules = s).catch(() => {}).finally(() => this.loadingSchedules = false); }
    loadStats() { this.api.getStats().then(s => this.diskStats = s).catch(() => {}); }

    getFileIcon(ext: string): string {
        const icons: Record<string, string> = { html: `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>`, htm: `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>`, css: `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 22l-2-18h20l-2 18-8 4-8-4z"/></svg>`, js: `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7l10 5 10-5-10-5z"/></svg>`, pdf: `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/></svg>` };
        return icons[ext] || `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/></svg>`;
    }
    formatSize(bytes: number): string { if (bytes < 1024) return bytes + 'B'; if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + 'KB'; return (bytes / (1024 * 1024)).toFixed(1) + 'MB'; }
    protected readonly statusClass = (s: string) => 'status-' + s;
    windowOpen(url: string) { window.open(url, '_blank'); }

    async exportJob(id: string) { try { const r = await fetch(`/api/jobs/${id}/export`, { method: 'POST' }); const d = await r.json(); const b = new Blob([JSON.stringify(d, null, 2)], { type: 'application/json' }); const u = URL.createObjectURL(b); const a = document.createElement('a'); a.href = u; a.download = `shinobi-export-${id.slice(0, 8)}.json`; a.click(); URL.revokeObjectURL(u); } catch { this.setStatus('Export failed', 'error'); } }
    async exportDeepResult(r: DeepResult) { try { const b = new Blob([JSON.stringify(r, null, 2)], { type: 'application/json' }); const u = URL.createObjectURL(b); const a = document.createElement('a'); a.href = u; a.download = `deep-${r.id.slice(0, 8)}.json`; a.click(); URL.revokeObjectURL(u); } catch {} }
    async exportDeepCsv() { try { const blob = await this.api.exportDeepCsv(); const u = URL.createObjectURL(blob); const a = document.createElement('a'); a.href = u; a.download = 'deep-results.csv'; a.click(); URL.revokeObjectURL(u); } catch {} }

    async exportDatabase() { try { const r = await fetch('/api/database/export'); const d = await r.json(); const b = new Blob([JSON.stringify(d, null, 2)], { type: 'application/json' }); const u = URL.createObjectURL(b); const a = document.createElement('a'); a.href = u; a.download = `shinobi-db-export-${new Date().toISOString().slice(0, 10)}.json`; a.click(); URL.revokeObjectURL(u); this.setStatus('DB exported'); } catch { this.setStatus('Export failed', 'error'); } }

    async importDatabase() {
        const input = document.createElement('input'); input.type = 'file'; input.accept = '.json';
        input.onchange = async () => {
            const file = input.files?.[0]; if (!file) return;
            try { const text = await file.text(); const data = JSON.parse(text); const r = await fetch('/api/database/import', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ jobs: data.jobs || [] }) }); if (r.ok) { this.setStatus('Database imported'); this.loadJobs(); } else this.setStatus('Import failed', 'error'); } catch { this.setStatus('Invalid JSON file', 'error'); }
        }; input.click();
    }

    selectDeepResult(r: DeepResult) { this.selectedDeepResult = r; if (r.structured_data) this.activeTab = 'deep'; }
    formatJson(obj: any): string { try { return JSON.stringify(obj, null, 2); } catch { return String(obj); } }
    getStructuredTypes(r: DeepResult): string[] { return r.structured_data ? Object.keys(r.structured_data) : []; }

    async deleteJob(id: string, event: Event) {
        event.stopPropagation();
        if (!confirm('Delete this job?')) return;
        try { await fetch(`/api/jobs/${id}`, { method: 'DELETE' }); this.jobs = this.jobs.filter(j => j.id !== id); if (this.activeJob?.id === id) { this.activeJob = null; this.stream?.close(); this.stream = null; } this.setStatus('Job deleted'); } catch { this.setStatus('Delete failed', 'error'); }
    }

    async deleteDeepResult(id: string, event: Event) {
        event.stopPropagation();
        if (!confirm('Delete this result?')) return;
        try { await fetch(`/api/deep/results/${id}`, { method: 'DELETE' }); this.deepResults = this.deepResults.filter(r => r.id !== id); if (this.selectedDeepResult?.id === id) this.selectedDeepResult = null; this.setStatus('Result deleted'); } catch { this.setStatus('Delete failed', 'error'); }
    }

    async clearDeepResults() {
        if (!confirm('Delete all deep research results?')) return; this.deleting = true;
        try { await fetch('/api/deep/results', { method: 'DELETE' }); this.deepResults = []; this.selectedDeepResult = null; this.setStatus('Deep results cleared'); } catch { this.setStatus('Clear failed', 'error'); } finally { this.deleting = false; }
    }

    async clearDatabase() {
        if (!confirm('Delete ALL jobs, deep results, and files?')) return; this.deleting = true;
        try { await fetch('/api/database/clear', { method: 'POST' }); this.jobs = []; this.deepResults = []; this.selectedDeepResult = null; this.activeJob = null; this.files = []; this.setStatus('Database cleared'); } catch { this.setStatus('Clear failed', 'error'); } finally { this.deleting = false; }
    }

    async addSchedule() {
        if (!this.newScheduleUrl.trim() || this.newScheduleInterval < 5) { this.setStatus('URL and interval >= 5 min required', 'warn'); return; }
        try { await this.api.createSchedule(this.newScheduleUrl.trim(), this.newScheduleInterval); this.newScheduleUrl = ''; this.newScheduleInterval = 60; this.loadSchedules(); this.setStatus('Schedule created'); } catch (e: any) { this.setStatus('Failed: ' + (e.message || ''), 'error'); }
    }

    async deleteSchedule(s: any) {
        if (!confirm('Delete schedule?')) return;
        try { await this.api.deleteSchedule(s.id); this.schedules = this.schedules.filter(x => x.id !== s.id); this.setStatus('Schedule deleted'); } catch { this.setStatus('Delete failed', 'error'); }
    }

    async previewFileItem(f: FileInfo) {
        this.previewFile = f; this.previewContent = null; this.previewLoading = true;
        try {
            const ext = f.name.split('.').pop()?.toLowerCase() || '';
            if (['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp'].includes(ext)) {
                this.previewContent = `<img src="/api/files/${f.path}" style="max-width:100%" />`;
            } else {
                const r = await fetch(`/api/files/${f.path}`);
                const text = await r.text();
                this.previewContent = text.length > 50000 ? text.slice(0, 50000) + '\n\n[... truncated ...]' : text;
            }
        } catch { this.previewContent = '[ERROR: Could not load file]'; }
        finally { this.previewLoading = false; }
    }
}
