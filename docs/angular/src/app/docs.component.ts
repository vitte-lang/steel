import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

import { AppState } from './app.state';

@Component({
  selector: 'app-docs',
  imports: [CommonModule],
  templateUrl: './docs.component.html'
})
export class DocsComponent {
  constructor(public state: AppState) {}

  get t() {
    return this.state.t();
  }

  get examples() {
    return this.state.examples;
  }
}
