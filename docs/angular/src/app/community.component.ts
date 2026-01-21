import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { AppState } from './app.state';

@Component({
  selector: 'app-community',
  imports: [CommonModule],
  templateUrl: './community.component.html'
})
export class CommunityComponent {
  constructor(public state: AppState) {}

  get t() {
    return this.state.t();
  }
}
