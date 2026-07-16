import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '../../../../pipes/translate.pipe';

@Component({
  selector: 'app-audit-export-actions',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './audit-export-actions.component.html',
  styleUrls: ['./audit-export-actions.component.scss'],
})
export class AuditExportActionsComponent {
  @Input() hasExports = false;
  @Input() findingCount = 0;

  @Output() exportCsv = new EventEmitter<void>();
  @Output() exportJson = new EventEmitter<void>();
  @Output() exportLighthouse = new EventEmitter<void>();
  @Output() exportPdf = new EventEmitter<void>();
  @Output() exportHtml = new EventEmitter<void>();
  @Output() exportMd = new EventEmitter<void>();
}
