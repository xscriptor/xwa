import { Injectable, inject } from '@angular/core';
import { ScrapeConfig, JobInfo, FileInfo, DeepConfig, DeepResult, DeepBatchQuery } from '../models/models';

@Injectable({ providedIn: 'root' })
export class ApiService {
    async startScrape(config: ScrapeConfig): Promise<JobInfo> {
        const body: any = { ...config };
        if (typeof body.file_types === 'string' && body.file_types.trim()) {
            body.file_types = body.file_types.split(',').map((s: string) => s.trim()).filter(Boolean);
        } else {
            delete body.file_types;
        }
        if (!body.auth_mode) { delete body.auth_username; delete body.auth_password; delete body.auth_mode; }
        const resp = await fetch('/api/scrape', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
        });
        if (!resp.ok) throw new Error(await resp.text());
        return resp.json();
    }

    async listJobs(offset = 0, limit = 50): Promise<{items: JobInfo[], total: number}> {
        const resp = await fetch(`/api/jobs?offset=${offset}&limit=${limit}`);
        return resp.json();
    }

    async cancelJob(id: string): Promise<void> {
        await fetch(`/api/jobs/${id}/cancel`, { method: 'POST' });
    }

    async listFiles(prefix = '', offset = 0, limit = 50): Promise<{items: FileInfo[], total: number}> {
        const resp = await fetch(`/api/files?prefix=${encodeURIComponent(prefix)}&offset=${offset}&limit=${limit}`);
        return resp.json();
    }

    jobStream(id: string): EventSource {
        return new EventSource(`/api/jobs/${id}/stream`);
    }

    async startDeepScrape(config: DeepConfig): Promise<DeepResult> {
        const resp = await fetch('/api/deep/scrape', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(config),
        });
        if (!resp.ok) throw new Error(await resp.text());
        return resp.json();
    }

    async startDeepBatch(config: DeepBatchQuery): Promise<DeepResult[]> {
        const resp = await fetch('/api/deep/batch', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(config),
        });
        if (!resp.ok) throw new Error(await resp.text());
        return resp.json();
    }

    async listDeepResults(offset = 0, limit = 50): Promise<{items: DeepResult[], total: number}> {
        const resp = await fetch(`/api/deep/results?offset=${offset}&limit=${limit}`);
        return resp.json();
    }

    async getStats(): Promise<any> {
        const resp = await fetch('/api/stats');
        return resp.json();
    }

    async searchFiles(q: string, offset = 0, limit = 50): Promise<any> {
        const resp = await fetch(`/api/search?q=${encodeURIComponent(q)}&offset=${offset}&limit=${limit}`);
        return resp.json();
    }

    async getDeepResult(id: string): Promise<DeepResult> {
        const resp = await fetch(`/api/deep/results/${id}`);
        return resp.json();
    }

    async deleteJob(id: string): Promise<void> {
        await fetch(`/api/jobs/${id}`, { method: 'DELETE' });
    }

    async deleteDeepResult(id: string): Promise<void> {
        await fetch(`/api/deep/results/${id}`, { method: 'DELETE' });
    }

    async clearDeepResults(): Promise<void> {
        await fetch('/api/deep/results', { method: 'DELETE' });
    }

    async clearDatabase(): Promise<void> {
        await fetch('/api/database/clear', { method: 'POST' });
    }

    async listSchedules(): Promise<any[]> {
        const resp = await fetch('/api/schedules');
        return resp.json();
    }

    async createSchedule(url: string, intervalMin: number): Promise<any> {
        const resp = await fetch('/api/schedules', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ url, interval_min: intervalMin }),
        });
        if (!resp.ok) throw new Error(await resp.text());
        return resp.json();
    }

    async deleteSchedule(id: string): Promise<void> {
        await fetch(`/api/schedules/${id}`, { method: 'DELETE' });
    }

    async exportDeepCsv(): Promise<Blob> {
        const resp = await fetch('/api/deep/results.csv');
        return resp.blob();
    }

    async startPythonCrawl(config: any): Promise<any> {
        const resp = await fetch('/api/deep/crawl', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(config),
        });
        if (!resp.ok) throw new Error(await resp.text());
        return resp.json();
    }

    async getCrawlStatus(jobId: string): Promise<any> {
        const resp = await fetch(`/api/deep/crawl/${jobId}/status`);
        if (!resp.ok) throw new Error('Not found');
        return resp.json();
    }

    async getCrawlResults(jobId: string): Promise<any> {
        const resp = await fetch(`/api/deep/crawl/${jobId}/results`);
        if (!resp.ok) throw new Error('Not found');
        return resp.json();
    }

    async cancelCrawl(jobId: string): Promise<void> {
        await fetch(`/api/deep/crawl/${jobId}/cancel`, { method: 'POST' });
    }
}
