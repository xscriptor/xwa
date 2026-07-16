import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '../../../../../../pipes/translate.pipe';

@Component({
  selector: 'app-findings-empty-state',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './findings-empty-state.component.html',
  styleUrls: ['./findings-empty-state.component.scss']
})
export class FindingsEmptyStateComponent {}
