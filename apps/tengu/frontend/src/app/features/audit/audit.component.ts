import { Component, OnInit, ChangeDetectorRef, ViewChild, ElementRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router, ActivatedRoute } from '@angular/router';
import { HttpClient } from '@angular/common/http';
import { AuditExportActionsComponent } from './components/export-actions/audit-export-actions.component';
import { TranslatePipe } from '../../pipes/translate.pipe';
import { TranslateService } from '../../services/translate.service';
import jsPDF from 'jspdf';
import autoTable from 'jspdf-autotable';

interface Finding {
  category: string;
  check: string;
  severity: string;
  title: string;
  description: string;
  snippet?: string;
  page_url?: string;
}

interface AuditRecord {
  id: string;
  url: string;
  status: string;
  findings: Finding[];
  created_at: string;
}

interface AuditPayload {
  mode: string;
  url: string;
  subdomains: boolean;
  timestamp: string;
  summary: { error: number; warning: number; info: number; pass: number; total: number };
  findings: Finding[];
}

type AuditMode = 'single' | 'fullsite' | 'batch';
type CheckTab = 'all' | 'performance' | 'seo' | 'accessibility' | 'best_practices';

@Component({
  selector: 'app-audit',
  standalone: true,
  imports: [CommonModule, FormsModule, AuditExportActionsComponent, TranslatePipe],
  templateUrl: './audit.component.html',
  styleUrls: ['./audit.component.scss'],
})
export class AuditComponent implements OnInit {
  @ViewChild('terminalLog') terminalLog!: ElementRef;
  targetUrl = '';
  auditMode: AuditMode = 'single';
  includeSubdomains = false;
  batchUrl = '';
  batchFormat = 'sitemap';
  checkTab: CheckTab = 'all';
  filterTab: string = 'all';
  filterCheck: string = 'all';
  loadedId: string | null = null;

  isRunning = false;
  isComplete = false;
  findings: Finding[] = [];
  pagesFound: string[] = [];
  pageHtml: string = '';
  htmlViewOpen = false;
  logs: string[] = [];

  private ws: WebSocket | null = null;

  private tr(key: string): string {
    return this.translate.t(key);
  }

  private findingTitleEs(f: Finding): string {
    const map: Record<string, string> = {
      'alt_text': 'Imagen(es) sin texto alternativo',
      'headings_outline': 'La estructura de encabezados salta uno o más niveles',
      'aria_roles': 'role=\'button\' en elemento no interactivo sin tabindex',
      'aria_usage': 'No se detectaron atributos ARIA en la página',
      'landmarks': 'No se encontraron regiones landmark',
      'form_labels': 'Control(es) de formulario sin etiqueta accesible',
      'keyboard_nav': 'Elemento(s) con valores positivos de tabindex',
      'link_text': 'El texto del enlace es genérico o no descriptivo',
      'tables': 'Falta <caption> en tabla de datos',
      'iframes': 'Iframe sin atributo title',
      'viewport': 'La meta etiqueta viewport impide el zoom',
      'lang_attribute': 'Falta atributo lang en <html>',
      'media_captions': 'Elemento(s) multimedia sin subtítulos',
      'color_contrast': 'Elemento(s) con contraste de color insuficiente',
      'color_contrast_bg_image': 'Elemento(s) con imagen de fondo — el contraste no se puede medir estáticamente',
      'focus_indicator': 'Elemento(s) enfocables con outline:none',
      'https': 'Página servida sobre HTTP inseguro',
      'security_headers': 'Falta cabecera de seguridad',
      'cookies': 'Cookie sin atributo Secure',
      'doctype': 'Falta o es incorrecta la declaración doctype',
      'deprecated_html': 'Elemento(s) HTML obsoleto(s)',
      'mixed_content': 'Contenido mixto detectado',
      'sri': 'Recurso(s) externo(s) sin integridad de subrecursos',
      'gdpr_consent': 'Mecanismo de consentimiento de cookies GDPR',
      'csp': 'La política de seguridad de contenido tiene problemas de configuración',
      'permissions_policy': 'Análisis de cabecera Permissions-Policy',
      'page_weight': 'El peso de la página excede lo recomendado',
      'image_audit': 'Problemas de optimización de imágenes',
      'font_audit': 'Problemas de carga de fuentes',
      'cache_header': 'Falta cabecera de caché',
      'compression': 'Falta compresión',
      'render_blocking': 'Recursos que bloquean el renderizado',
      'third_party_scripts': 'Scripts de terceros detectados',
      'web_vitals': 'LCP, CLS y INP requieren un navegador',
      'title': 'Problema con la etiqueta title',
      'meta_description': 'Problema con la meta descripción',
      'heading': 'Problema con la jerarquía de encabezados',
      'canonical': 'Problema con la URL canónica',
      'open_graph': 'Problema con etiquetas Open Graph',
      'twitter_card': 'Problema con Twitter Card',
      'json_ld': 'Problema con datos estructurados JSON-LD',
      'meta_robots': 'Problema con la etiqueta meta robots',
      'hreflang': 'Problema con etiquetas hreflang',
      'robots_txt': 'Problemas de configuración en robots.txt',
      'sitemap': 'Problema con el sitemap',
      'broken_links': 'Enlace(s) rotos encontrados',
      'redirect_chain': 'Cadena de redirecciones detectada',
      'structured_data_microdata': 'Datos estructurados Microdata/RDFa',
      'console_errors': 'La detección de errores de consola requiere un navegador',
    };
    return map[f.check] || f.title;
  }

  constructor(
    private cdr: ChangeDetectorRef,
    private router: Router,
    private route: ActivatedRoute,
    private http: HttpClient,
    private translate: TranslateService,
  ) {}

  ngOnInit(): void {
    this.route.queryParams.subscribe(params => {
      const id = params['load'];
      if (id) {
        this.loadAudit(id);
      }
    });
  }

  loadAudit(id: string): void {
    this.loadedId = id;
    this.http.get<AuditRecord>(`/api/audits/${id}`).subscribe({
      next: (record) => {
        this.targetUrl = record.url;
        this.findings = record.findings;
        this.isComplete = true;
        this.filterTab = 'all';
        this.filterCheck = 'all';
        this.cdr.detectChanges();
      },
      error: () => {
        this.logs.push('[!] Failed to load audit');
        this.cdr.detectChanges();
      },
    });
  }

  get htmlLines(): string[] {
    return this.pageHtml.split('\n');
  }

  toggleHtmlView(): void {
    this.htmlViewOpen = !this.htmlViewOpen;
  }

  formatHtmlLine(line: string): string {
    const escaped = line
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
    return escaped
      .replace(/(&lt;\/?)([a-zA-Z][\w-]*)/g, '<span class="syn-tag">$1$2</span>')
      .replace(/\b((?:src|href|class|id|name|rel|type|alt|title|width|height|lang|charset|content|property|http-equiv|media|style|integrity|crossorigin|as|hreflang|fetchPriority|noModule|aria-label|aria-current|aria-expanded|aria-controls|aria-busy|aria-live|aria-hidden|aria-describedby|aria-labelledby|target|rel|sizes))\s*=\s*/g, '<span class="syn-attr">$1</span>=')
      .replace(/=("(?:[^"\\]|\\.)*")/g, '=<span class="syn-val">$1</span>')
      .replace(/(&lt;\/)([a-zA-Z][\w-]*)(&gt;)/g, '<span class="syn-tag">$1$2</span>$3')
      .replace(/(&lt;)([a-zA-Z][\w-]*)/g, '<span class="syn-tag">$1$2</span>')
      .replace(/(\/?&gt;)/g, '<span class="syn-punc">$1</span>');
  }

  private snippetKeys(snippet: string): string[] {
    const keys: string[] = [];
    const attrRx = /(?:src|href|id|name|for|alt|aria-label|title)=["']([^"']+?)["']/g;
    let m: RegExpExecArray | null;
    while ((m = attrRx.exec(snippet)) !== null) {
      keys.push(m[1]);
    }
    return keys;
  }

  private snippetTag(snippet: string): string | null {
    const m = snippet.match(/<([a-zA-Z][\w-]*)/);
    return m ? m[1].toLowerCase() : null;
  }

  hasLineHighlight(lineIdx: number): boolean {
    if (!this.pageHtml) return false;
    const line = this.htmlLines[lineIdx];
    if (!line) return false;
    for (const f of this.findings) {
      if (!f.snippet) continue;
      const tag = this.snippetTag(f.snippet);
      if (tag && !line.includes(`<${tag}`) && !line.includes(`</${tag}>`)) continue;
      const keys = this.snippetKeys(f.snippet);
      if (keys.length > 0 && keys.some(k => line.includes(k))) return true;
      if (line.includes(f.snippet.slice(0, 80))) return true;
      const normLine = line.replace(/\s+/g, ' ');
      const normSnip = f.snippet.replace(/\s+/g, ' ').trim();
      if (normLine.includes(normSnip.slice(0, 100))) return true;
    }
    return false;
  }

  findingLine(finding: Finding): number {
    if (!this.pageHtml || !finding.snippet) return -1;
    const lines = this.htmlLines;
    const tag = this.snippetTag(finding.snippet);
    const keys = this.snippetKeys(finding.snippet);

    const norm = (s: string) => s.replace(/\s+/g, ' ').trim();
    const snipNorm = norm(finding.snippet);

    if (tag) {
      for (let i = 0; i < lines.length; i++) {
        if (!lines[i].includes(`<${tag}`)) continue;
        if (keys.some(k => lines[i].includes(k))) return i + 1;
        if (norm(lines[i]).includes(snipNorm.slice(0, 80))) return i + 1;
      }
      for (let i = 0; i < lines.length; i++) {
        if (lines[i].includes(`<${tag}`)) return i + 1;
      }
    }

    for (let i = 0; i < lines.length; i++) {
      if (norm(lines[i]).includes(snipNorm.slice(0, 100))) return i + 1;
    }

    return -1;
  }

  get severityColor(): Record<string, string> {
    return { Error: '#D71921', Warning: '#D4A843', Info: '#5B9BF6', Pass: '#4A9E5C' };
  }

  get hasCrawlOptions(): boolean {
    return this.auditMode === 'fullsite';
  }

  get hasBatchOptions(): boolean {
    return this.auditMode === 'batch';
  }

  get selectedChecks(): string[] {
    return ['performance', 'seo', 'accessibility', 'best_practices'];
  }

  setCheckTab(tab: CheckTab): void {
    this.checkTab = tab;
    this.filterTab = tab === 'all' ? 'all' : tab;
    this.filterCheck = 'all';
    this.cdr.detectChanges();
  }

  get checkTypes(): string[] {
    const cats = this.filterTab === 'all' ? [] : this.findings.filter(f => f.category === this.filterTab).map(f => f.check);
    return [...new Set(cats)];
  }

  get filteredFindings(): Finding[] {
    let result = this.findings;
    if (this.filterTab !== 'all') {
      result = result.filter((f) => f.category === this.filterTab);
    }
    if (this.filterCheck !== 'all') {
      result = result.filter((f) => f.check === this.filterCheck);
    }
    return result;
  }

  setFilterTab(tab: string): void {
    this.filterTab = tab;
    this.filterCheck = 'all';
    if (tab === 'all' || tab === 'performance' || tab === 'seo' || tab === 'accessibility' || tab === 'best_practices') {
      this.checkTab = tab as CheckTab;
    }
    this.cdr.detectChanges();
  }

  setCheckFilter(check: string): void {
    this.filterCheck = check;
    this.cdr.detectChanges();
  }

  labelCheck(check: string): string {
    return check.replace(/_/g, ' ');
  }

  checkCount(check: string): number {
    return this.findings.filter(f => f.category === this.filterTab && f.check === check).length;
  }

  private scrollTerminal(): void {
    setTimeout(() => {
      if (this.terminalLog) {
        this.terminalLog.nativeElement.scrollTop = this.terminalLog.nativeElement.scrollHeight;
      }
    }, 0);
  }

  startAudit(): void {
    if (!this.targetUrl.trim()) return;
    if (!this.targetUrl.match(/^https?:\/\//)) {
      this.targetUrl = 'https://' + this.targetUrl;
    }

    this.loadedId = null;
    this.isRunning = true;
    this.isComplete = false;
    this.findings = [];
    this.pagesFound = [];
    this.pageHtml = '';
    this.logs = [];
    this.checkTab = 'all';
    this.filterTab = 'all';
    this.filterCheck = 'all';

    if (this.auditMode === 'batch') {
      this.batchUrl = this.targetUrl;
    }

    this.cdr.detectChanges();

    const params = new URLSearchParams({
      url: this.auditMode === 'batch' ? this.batchUrl : this.targetUrl,
      mode: this.auditMode,
      subdomains: String(this.includeSubdomains),
      checks: this.selectedChecks.join(','),
    });

    if (this.auditMode === 'batch') {
      params.set('batch_url', this.batchUrl);
      params.set('batch_format', this.batchFormat);
    }

    this.ws = new WebSocket(`/api/audit/live?${params}`);

    this.ws.onmessage = (event) => {
      const msg = event.data;

      if (msg.startsWith('[HTML]')) {
        this.pageHtml = msg.slice(6);
      } else if (msg.startsWith('[AUDIT_META]')) {
        this.logs.push(msg);
      } else       if (msg.startsWith('[AUDIT]')) {
        this.logs.push(msg);
        this.scrollTerminal();
      } else if (msg.startsWith('[AUDIT_META]')) {
        this.logs.push(msg);
        this.scrollTerminal();
      } else if (msg.startsWith('[PAGE]')) {
        const url = msg.slice(6);
        this.pagesFound.push(url);
      } else if (msg.startsWith('[done]')) {
        this.logs.push(msg);
        this.isRunning = false;
        this.isComplete = true;
      } else if (msg.startsWith('[!]')) {
        this.logs.push(msg);
        this.isRunning = false;
        this.isComplete = true;
      } else {
        try {
          const finding: Finding = JSON.parse(msg);
          this.findings.push(finding);
        } catch {
          this.logs.push(`[RAW] ${msg}`);
        }
      }
      this.cdr.detectChanges();
    };

    this.ws.onerror = () => {
      this.logs.push('[!] WebSocket connection error');
      this.isRunning = false;
      this.isComplete = true;
      this.cdr.detectChanges();
    };

    this.ws.onclose = () => {
      this.isRunning = false;
      this.isComplete = true;
      this.cdr.detectChanges();
    };
  }

  getFindingsByCategory(category: string): Finding[] {
    return this.findings.filter((f) => f.category === category);
  }

  getFindingsBySeverity(severity: string): Finding[] {
    return this.findings.filter((f) => f.severity === severity);
  }

  get categories(): string[] {
    return [...new Set(this.findings.map((f) => f.category))];
  }

  get categoryCounts(): Record<string, number> {
    const counts: Record<string, number> = {};
    for (const f of this.findings) {
      counts[f.category] = (counts[f.category] || 0) + 1;
    }
    return counts;
  }

  get summary(): { error: number; warning: number; info: number; pass: number; total: number } {
    return {
      error: this.getFindingsBySeverity('Error').length,
      warning: this.getFindingsBySeverity('Warning').length,
      info: this.getFindingsBySeverity('Info').length,
      pass: this.getFindingsBySeverity('Pass').length,
      total: this.findings.length,
    };
  }

  /* ---- Export Logic ---- */

  private buildPayload(findings?: Finding[]): AuditPayload {
    const f = findings ?? this.findings;
    return {
      mode: this.auditMode,
      url: this.auditMode === 'batch' ? this.batchUrl : this.targetUrl,
      subdomains: this.includeSubdomains,
      timestamp: new Date().toISOString(),
      summary: {
        error: f.filter(x => x.severity === 'Error').length,
        warning: f.filter(x => x.severity === 'Warning').length,
        info: f.filter(x => x.severity === 'Info').length,
        pass: f.filter(x => x.severity === 'Pass').length,
        total: f.length,
      },
      findings: f,
    };
  }

  private downloadBlob(blob: Blob, filename: string): void {
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = filename;
    anchor.click();
    URL.revokeObjectURL(url);
  }

  categoryLabel(cat: string): string {
    return cat.replace(/_/g, '-').toUpperCase();
  }

  private exportTitle(f: Finding): string {
    if (this.translate.isEn()) return f.title;
    return this.findingTitleEs(f);
  }

  private exportDescription(f: Finding): string {
    if (this.translate.isEn()) return f.description;
    const map: Record<string, string> = {
      'color_contrast': 'Se encontraron elementos con contraste de color insuficiente entre el texto y el fondo. tamaños de texto grandes (≥18pt o ≥14pt bold): 3:1.',
      'color_contrast_bg_image': 'Se encontraron elementos de texto con una imagen de fondo CSS. El análisis estático no puede medir el contraste.',
      'https': 'La página se sirve sobre HTTP sin cifrar. Esto es un problema crítico de seguridad y SEO.',
      'alt_text': 'Se encontraron imágenes sin atributo alt. Los lectores de pantalla no pueden describir el contenido.',
      'headings_outline': 'La página no tiene elementos de encabezado (h1–h6) o la jerarquía es incorrecta.',
      'form_labels': 'Se encontraron controles de formulario sin etiqueta accesible.',
      'landmarks': 'La página no usa elementos landmark HTML5 o roles ARIA landmark.',
      'viewport': 'La etiqueta meta viewport impide el zoom en dispositivos móviles.',
      'lang_attribute': 'El elemento <html> no tiene atributo lang.',
      'focus_indicator': 'Se encontraron elementos enfocables con outline:none.',
      'broken_links': 'Se encontraron enlaces rotos en la página.',
      'redirect_chain': 'La URL pasa por múltiples redirecciones antes de llegar al destino final.',
    };
    return map[f.check] || f.title;
  }

  exportCsv(): void {
    const items = this.filteredFindings;
    const headers = ['mode', 'url', 'page_url', 'category', 'check', 'severity', 'title', 'description', 'snippet', 'line'];
    const escape = (v: string) => `"${String(v || '').replace(/"/g, '""').replace(/\n/g, '\\n').replace(/\r/g, '\\r')}"`;
    const rows = items.map((f) =>
      headers.map((h) => {
        if (h === 'mode') return escape(this.auditMode);
        if (h === 'url') return escape(this.targetUrl);
        if (h === 'page_url') return escape(f.page_url || this.targetUrl);
        if (h === 'title') return escape(this.exportTitle(f));
        if (h === 'description') return escape(this.exportDescription(f));
        if (h === 'line') return escape(String(this.findingLine(f)));
        return escape((f as any)[h]);
      }).join(',')
    );
    const footer = `\n# ${this.tr('audit.generated_by')}`;
    const csv = [headers.join(','), ...rows, footer].join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    this.downloadBlob(blob, `tengu-audit-${this.targetUrl.replace(/[^a-z0-9]/gi, '-')}.csv`);
  }

  exportJson(): void {
    const items = this.filteredFindings;
    const payload = this.buildPayload(items);
    const enriched = {
      ...payload,
      generator: this.tr('audit.generated_by'),
      findings: items.map(f => ({ ...f, line: this.findingLine(f), page_url: f.page_url || this.targetUrl })),
    };
    const json = JSON.stringify(enriched, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    this.downloadBlob(blob, `tengu-audit-${this.targetUrl.replace(/[^a-z0-9]/gi, '-')}.json`);
  }

  exportLighthouse(): void {
    const items = this.filteredFindings;
    const lhr: any = {
      lighthouseVersion: '11.0.0',
      requestedUrl: this.targetUrl,
      finalUrl: this.targetUrl,
      fetchTime: new Date().toISOString(),
      userAgent: 'Tengu',
      environment: { networkUserAgent: 'Tengu', benchmarkIndex: 1 },
      configSettings: {
        formFactor: 'desktop',
        screenEmulation: { mobile: false, width: 1350, height: 940, deviceScaleFactor: 1 },
      },
      categories: {
        performance: { title: 'Performance', score: null },
        seo: { title: 'SEO', score: null },
        accessibility: { title: 'Accessibility', score: null },
        'best-practices': { title: 'Best Practices', score: null },
      },
      categoryGroups: {},
      audits: {} as Record<string, any>,
    };

    const catMap: Record<string, string> = {
      performance: 'performance',
      seo: 'seo',
      accessibility: 'accessibility',
      best_practices: 'best-practices',
    };

    for (const f of items) {
      const auditId = f.check;
      const score = f.severity === 'Error' ? 0 : f.severity === 'Warning' ? 0.5 : f.severity === 'Pass' ? 1 : null;
      lhr.audits[auditId] = {
        id: auditId,
        title: f.title,
        description: f.description,
        score: score,
        scoreDisplayMode: score === null ? 'informative' : 'binary',
        numericValue: null,
        displayValue: null,
        warnings: [],
        details: { items: [{ snippet: f.snippet || '', page_url: f.page_url || this.targetUrl }] },
      };
    }

    const auditIds = Object.keys(lhr.audits);
    for (const [catKey, lhCat] of Object.entries(lhr.categories)) {
      const mapped = lhCat as any;
      const catFindings = items.filter(f => {
        const mappedCat = catMap[f.category];
        return mappedCat === catKey;
      });
      if (catFindings.length > 0) {
        const errors = catFindings.filter(f => f.severity === 'Error').length;
        const warnings = catFindings.filter(f => f.severity === 'Warning').length;
        const passes = catFindings.filter(f => f.severity === 'Pass').length;
        const total = catFindings.length;
        const score = total > 0 ? Math.max(0, 1 - (errors * 0.3 + warnings * 0.1)) : null;
        mapped.score = score;
        mapped.manualDescription = '';
        mapped.auditRefs = catFindings.map(f => ({
          id: f.check,
          weight: 1,
          group: '',
        }));
      }
    }

    const json = JSON.stringify(lhr, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    this.downloadBlob(blob, `tengu-lighthouse-${this.targetUrl.replace(/[^a-z0-9]/gi, '-')}.json`);
  }

  exportPdf(): void {
    const items = this.filteredFindings;
    const summary = {
      error: items.filter(x => x.severity === 'Error').length,
      warning: items.filter(x => x.severity === 'Warning').length,
      info: items.filter(x => x.severity === 'Info').length,
      pass: items.filter(x => x.severity === 'Pass').length,
      total: items.length,
    };
    const doc = new jsPDF('landscape', 'mm', 'a4');
    doc.setFillColor(0, 0, 0);
    doc.rect(0, 0, 297, 210, 'F');
    doc.setTextColor(255, 255, 255);
    doc.setFont('helvetica', 'bold');
    doc.setFontSize(18);
    doc.text(this.tr('audit.report_title'), 15, 25);
    doc.setFontSize(7);
    doc.setTextColor(153, 153, 153);
    doc.text('— XWA submodule — Xscriptor', 15, 30);
    doc.setFont('helvetica', 'normal');
    doc.setTextColor(255, 255, 255);
    doc.setFontSize(10);
    doc.text(`${this.tr('audit.url')}: ${this.targetUrl}  (${this.tr('audit.mode_' + this.auditMode)})`, 15, 38);
    doc.text(`${this.tr('audit.date')}: ${new Date().toISOString()}  ${this.tr('audit.findings')}: ${summary.total}`, 15, 44);

    const headers = [[this.tr('audit.category'), this.tr('audit.check'), this.tr('audit.sev'), this.tr('audit.issue'), this.tr('audit.page'), 'L', this.tr('audit.code')]];
    const maxCodeLen = 80;
    const data = items.map((f) => [
      this.categoryLabel(f.category),
      f.check,
      f.severity.toUpperCase().slice(0, 4),
      this.exportTitle(f),
      (f.page_url || this.targetUrl).replace(/^https?:\/\//, '').slice(0, 30),
      this.findingLine(f) > 0 ? 'L' + this.findingLine(f) : '-',
      (f.snippet || this.exportDescription(f)).slice(0, maxCodeLen).replace(/\s+/g, ' ').trim(),
    ]);

    autoTable(doc, {
      startY: 50,
      head: headers,
      body: data,
      theme: 'grid',
      styles: { fillColor: [17, 17, 17], textColor: [232, 232, 232], fontSize: 7, font: 'helvetica', cellPadding: 2 },
      headStyles: { fillColor: [0, 0, 0], textColor: [232, 232, 232], fontStyle: 'bold' },
      alternateRowStyles: { fillColor: [26, 26, 26] },
      columnStyles: {
        0: { cellWidth: 16 },
        1: { cellWidth: 22 },
        2: { cellWidth: 10 },
        3: { cellWidth: 44 },
        4: { cellWidth: 36 },
        5: { cellWidth: 8 },
        6: { cellWidth: 'auto' },
      },
    });

    doc.save(`tengu-audit-${this.targetUrl.replace(/[^a-z0-9]/gi, '-')}.pdf`);
  }

  exportHtml(): void {
    const items = this.filteredFindings;
    const summary = {
      error: items.filter(x => x.severity === 'Error').length,
      warning: items.filter(x => x.severity === 'Warning').length,
      info: items.filter(x => x.severity === 'Info').length,
      pass: items.filter(x => x.severity === 'Pass').length,
      total: items.length,
    };
    const sevColors: Record<string, string> = {
      error: '#D71921', warning: '#D4A843', info: '#5B9BF6', pass: '#4A9E5C',
    };
    const catColors: Record<string, string> = {
      performance: '#5B9BF6', seo: '#D4A843', accessibility: '#4A9E5C', best_practices: '#D71921',
    };
    const lang = this.translate.isEn() ? 'en' : 'es';

    const findingsHtml = items
      .map((f) => `<tr>
        <td><span style="color:${catColors[f.category] || '#999'}">${this.categoryLabel(f.category)}</span></td>
        <td><code>${f.check}</code></td>
        <td><span style="color:${sevColors[f.severity.toLowerCase()] || '#999'};font-family:'Space Mono',monospace;font-size:11px">${f.severity.toUpperCase()}</span></td>
        <td>${this.exportTitle(f)}</td>
        <td style="font-size:12px;color:#5B9BF6;font-family:'Space Mono',monospace;word-break:break-all">${f.page_url ? `<a href="${f.page_url}" style="color:#5B9BF6">${f.page_url}</a>` : '-'}</td>
        <td style="font-size:12px;color:#999;white-space:pre-wrap;max-width:400px">${this.exportDescription(f)}</td>
        <td style="font-size:12px;color:#666;font-family:'Space Mono',monospace">${this.findingLine(f) > 0 ? 'L' + this.findingLine(f) : '-'}</td>
      </tr>`)
      .join('\n');

    const html = `<!DOCTYPE html>
<html lang="${lang}">
<head><meta charset="utf-8"><title>${this.tr('audit.report_title')}</title>
<style>
  body { background:#000; color:#E8E8E8; font-family:'Space Grotesk',sans-serif; padding:40px; }
  h1 { font-size:24px; letter-spacing:-0.01em; margin:0 0 2px; }
  .sig { font-size:9px; color:#666; margin:0 0 20px; letter-spacing:0.02em; }
  .meta { color:#999; font-size:14px; margin-bottom:30px; }
  .meta span { margin-right:20px; }
  .summary { display:flex; gap:24px; margin-bottom:30px; }
  .stat { text-align:center; }
  .stat .num { font-size:36px; font-family:'Space Mono',monospace; }
  .stat .lbl { font-size:11px; text-transform:uppercase; letter-spacing:0.08em; color:#999; }
  table { width:100%; border-collapse:collapse; }
  th { text-align:left; font-size:11px; text-transform:uppercase; letter-spacing:0.08em; color:#999; border-bottom:1px solid #333; padding:12px 8px; }
  td { padding:10px 8px; border-bottom:1px solid #222; font-size:14px; }
  code { font-family:'Space Mono',monospace; font-size:12px; color:#666; }
  a { color:#5B9BF6; }
  .footer { margin-top:40px; padding-top:20px; border-top:1px solid #222; color:#666; font-size:12px; }
</style></head>
<body>
  <h1>${this.tr('audit.report_title')}</h1>
  <div class="sig">— XWA submodule — Xscriptor</div>
  <div class="meta">
    <span>${this.tr('audit.url')}: ${this.targetUrl}</span>
    <span>${this.tr('audit.mode')}: ${this.tr('audit.mode_' + this.auditMode)}</span>
    <span>${this.tr('audit.date')}: ${new Date().toISOString()}</span>
    <span>${this.tr('audit.findings')}: ${summary.total}</span>
  </div>
  <div class="summary">
    <div class="stat">    <div class="num" style="color:#D71921">${summary.error}</div><div class="lbl">${this.tr('audit.errors')}</div></div>
    <div class="stat"><div class="num" style="color:#D4A843">${summary.warning}</div><div class="lbl">${this.tr('audit.warnings')}</div></div>
    <div class="stat"><div class="num" style="color:#5B9BF6">${summary.info}</div><div class="lbl">${this.tr('audit.info')}</div></div>
    <div class="stat"><div class="num" style="color:#4A9E5C">${summary.pass}</div><div class="lbl">${this.tr('audit.passed')}</div></div>
  </div>
  <table><thead><tr><th>${this.tr('audit.category')}</th><th>${this.tr('audit.check')}</th><th>${this.tr('audit.severity')}</th><th>${this.tr('audit.col_title')}</th><th>${this.tr('audit.page')}</th><th>${this.tr('audit.description')}</th><th>${this.tr('audit.line')}</th></tr></thead>
  <tbody>${findingsHtml}</tbody></table>
  <div class="footer">${this.tr('audit.generated_by')}</div>
</body></html>`;

    const blob = new Blob([html], { type: 'text/html' });
    this.downloadBlob(blob, `tengu-audit-${this.targetUrl.replace(/[^a-z0-9]/gi, '-')}.html`);
  }

  exportMd(): void {
    const items = this.filteredFindings;
    const summary = {
      error: items.filter(x => x.severity === 'Error').length,
      warning: items.filter(x => x.severity === 'Warning').length,
      info: items.filter(x => x.severity === 'Info').length,
      pass: items.filter(x => x.severity === 'Pass').length,
      total: items.length,
    };
    const lines: string[] = [];
    const sep = `---`;
    const url = this.targetUrl;

    lines.push(`# ${this.tr('audit.report_title')}`);
    lines.push(``);
    lines.push(`*— XWA submodule — Xscriptor*`);
    lines.push(``);
    lines.push(`**${this.tr('audit.url')}:** ${url}`);
    lines.push(`**${this.tr('audit.mode')}:** ${this.tr('audit.mode_' + this.auditMode)}`);
    lines.push(`**${this.tr('audit.date')}:** ${new Date().toISOString()}`);
    lines.push(`**${this.tr('audit.total_findings')}:** ${summary.total}`);
    lines.push(``);
    lines.push(sep);
    lines.push(``);
    lines.push(`## ${this.tr('audit.summary')}`);
    lines.push(``);
    lines.push(`| ${this.tr('audit.severity')} | ${this.tr('audit.count')} |`);
    lines.push(`|----------|-------|`);
    lines.push(`| ${this.tr('audit.error')}    | ${summary.error} |`);
    lines.push(`| ${this.tr('audit.warning')}  | ${summary.warning} |`);
    lines.push(`| ${this.tr('audit.info')}     | ${summary.info} |`);
    lines.push(`| ${this.tr('audit.pass')}     | ${summary.pass} |`);
    lines.push(``);
    lines.push(sep);
    lines.push(``);
    lines.push(`## ${this.tr('audit.findings')}`);
    lines.push(``);
    lines.push(`| ${this.tr('audit.category')} | ${this.tr('audit.check')} | ${this.tr('audit.severity')} | ${this.tr('audit.col_title')} | ${this.tr('audit.page')} | ${this.tr('audit.line')} |`);
    lines.push(`|----------|-------|----------|-------|------|------|`);

    for (const f of items) {
      const cat = this.categoryLabel(f.category);
      const title = this.exportTitle(f).replace(/\|/g, '\\|');
      const pageUrl = (f.page_url || this.targetUrl).replace(/^https?:\/\//, '').replace(/\|/g, '\\|');
      const ln = this.findingLine(f);
      lines.push(`| ${cat} | \`${f.check}\` | ${f.severity.toUpperCase()} | ${title} | ${pageUrl} | ${ln > 0 ? ln : '-'} |`);
    }

    lines.push(``);
    lines.push(sep);
    lines.push(``);
    lines.push(`*${this.tr('audit.generated_by')}*`);

    const md = lines.join('\n');
    const blob = new Blob([md], { type: 'text/markdown' });
    this.downloadBlob(blob, `tengu-audit-${url.replace(/[^a-z0-9]/gi, '-')}.md`);
  }
}
