import { CommonModule } from '@angular/common';
import { Component, Input } from '@angular/core';
import { TranslatePipe } from '../../../../pipes/translate.pipe';

@Component({
  selector: 'app-recon-terminal',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './recon-terminal.component.html',
  styleUrl: './recon-terminal.component.scss'
})
export class ReconTerminalComponent {
  @Input() lines: string[] = [];
  @Input() isScanning = false;
}
