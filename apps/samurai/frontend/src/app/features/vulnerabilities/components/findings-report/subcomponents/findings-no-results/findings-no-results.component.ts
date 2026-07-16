import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '../../../../../../pipes/translate.pipe';

@Component({
  selector: 'app-findings-no-results',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './findings-no-results.component.html',
  styleUrls: ['./findings-no-results.component.scss']
})
export class FindingsNoResultsComponent {}
