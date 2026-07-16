import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
  name: 't',
  standalone: true,
  pure: false
})
export class TranslatePipe implements PipeTransform {
  transform(value: string): string {
    return value;
  }
}
