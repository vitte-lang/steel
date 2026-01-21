import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { ApiService } from './services/api.service';
import { AppState } from './app.state';
import { SupabaseService } from './services/supabase.service';

@Component({
  selector: 'app-community',
  imports: [CommonModule, FormsModule],
  templateUrl: './community.component.html'
})
export class CommunityComponent {
  messageEmail = '';
  messageText = '';
  messageWebsite = '';
  messageTs = Date.now();
  messageStatus = '';

  constructor(
    public state: AppState,
    private api: ApiService,
    private auth: SupabaseService
  ) {}

  get t() {
    return this.state.t();
  }

  async submitMessage(): Promise<void> {
    this.messageStatus = '';
    const email = this.messageEmail || (await this.auth.getUserEmail()) || undefined;
    try {
      await this.api.postMessage({
        page: '/community',
        email,
        message: this.messageText,
        meta: { hp: this.messageWebsite, ts: this.messageTs }
      });
      this.messageText = '';
      this.messageTs = Date.now();
      this.messageStatus = 'Message sent.';
    } catch (err: any) {
      this.messageStatus = err.message || 'Failed to send message';
    }
  }
}
