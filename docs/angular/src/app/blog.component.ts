import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

import { AppState } from './app.state';

@Component({
  selector: 'app-blog',
  imports: [CommonModule, RouterLink],
  templateUrl: './blog.component.html'
})
export class BlogComponent {
  constructor(public state: AppState) {}

  get t() {
    return this.state.t();
  }
}
