import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '../../../../../../pipes/translate.pipe';
import { Finding } from '../../../../models/vulnerabilities.models';

@Component({
  selector: 'app-finding-item',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './finding-item.component.html',
  styleUrls: ['./finding-item.component.scss']
})
export class FindingItemComponent {
  @Input() finding!: Finding;
}
