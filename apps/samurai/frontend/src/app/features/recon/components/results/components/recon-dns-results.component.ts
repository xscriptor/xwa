import { CommonModule } from '@angular/common';
import { Component, Input } from '@angular/core';
import { TranslatePipe } from '../../../../../pipes/translate.pipe';

@Component({
  selector: 'app-recon-dns-results',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './recon-dns-results.component.html',
  styleUrl: './recon-dns-results.component.scss'
})
export class ReconDnsResultsComponent {
  @Input() dns: Record<string, string[]> | undefined;

  entries(): Array<[string, string[]]> {
    return Object.entries(this.dns || {});
  }
}
