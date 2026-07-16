import { CommonModule } from '@angular/common';
import { Component, EventEmitter, Input, Output } from '@angular/core';
import { TranslatePipe } from '../../../../pipes/translate.pipe';

@Component({
  selector: 'app-recon-export-actions',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './recon-export-actions.component.html',
  styleUrls: ['./recon-export-actions.component.scss']
})
export class ReconExportActionsComponent {
  @Input() hasExports = false;

  @Output() exportCsv = new EventEmitter<void>();
  @Output() exportJson = new EventEmitter<void>();
  @Output() exportPdf = new EventEmitter<void>();
  @Output() exportBinary = new EventEmitter<void>();
}
