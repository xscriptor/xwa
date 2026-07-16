import { CommonModule } from '@angular/common';
import { Component, Input } from '@angular/core';
import { TranslatePipe } from '../../../../../pipes/translate.pipe';

@Component({
  selector: 'app-recon-headers-results',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './recon-headers-results.component.html',
  styleUrl: './recon-headers-results.component.scss'
})
export class ReconHeadersResultsComponent {
  @Input() headers:
    | {
        present: Record<string, { value: string; description: string }>;
        missing: string[];
        risk_level: 'LOW' | 'MEDIUM' | 'HIGH';
        recommendations: string[];
      }
    | undefined;

  entries(): Array<[string, { value: string; description: string }]> {
    return Object.entries(this.headers?.present || {});
  }
}
