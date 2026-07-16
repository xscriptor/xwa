import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '../../../../pipes/translate.pipe';

@Component({
  selector: 'app-scanner-history',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './scanner-history.component.html',
  styleUrls: ['./scanner-history.component.scss']
})
export class ScannerHistoryComponent {
  @Input() scannerHistory: Array<{
    id: number;
    target: string;
    status: string;
    openPorts: number;
    contacts: number;
    unsanitized: number;
    createdAt: string;
  }> = [];

  @Input() selectedScanId: number | null = null;
  @Output() selectScan = new EventEmitter<number>();

  onSelectScan(scanId: number) {
    this.selectScan.emit(scanId);
  }
}
