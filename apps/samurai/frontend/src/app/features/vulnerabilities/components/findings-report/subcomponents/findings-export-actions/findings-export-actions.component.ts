import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '../../../../../../pipes/translate.pipe';

@Component({
  selector: 'app-findings-export-actions',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './findings-export-actions.component.html',
  styleUrls: ['./findings-export-actions.component.scss']
})
export class FindingsExportActionsComponent {
  @Input() findingsCount = 0;

  @Output() exportCsv = new EventEmitter<void>();
  @Output() exportJson = new EventEmitter<void>();
  @Output() exportPdf = new EventEmitter<void>();
  @Output() exportBinary = new EventEmitter<void>();
}
