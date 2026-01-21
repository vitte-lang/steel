import { CommonModule } from '@angular/common';
import { Component, OnInit } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';

import { ApiService } from './services/api.service';
import { AppState } from './app.state';
import { SupabaseService } from './services/supabase.service';

@Component({
  selector: 'app-blog-detail',
  imports: [CommonModule, FormsModule],
  templateUrl: './blog-detail.component.html'
})
export class BlogDetailComponent implements OnInit {
  post: any;
  comments: any[] = [];
  status = '';
  authStatus = '';
  authEmail = '';
  authPassword = '';
  form = {
    author: '',
    rating: 5,
    message: '',
    website: '',
    ts: Date.now()
  };
  email: string | null = null;

  constructor(
    private route: ActivatedRoute,
    private state: AppState,
    private api: ApiService,
    private auth: SupabaseService
  ) {}

  get t() {
    return this.state.t();
  }

  async ngOnInit(): Promise<void> {
    const slug = this.route.snapshot.paramMap.get('slug');
    this.post = this.t.blog.posts.find((p: any) => p.slug === slug);
    await this.refreshAuth();
    if (this.post) {
      await this.loadComments(this.post.slug);
      await this.api.trackPage(`/blog/${this.post.slug}`);
    }
  }

  async refreshAuth(): Promise<void> {
    this.email = await this.auth.getUserEmail();
    this.form.author = this.email || '';
  }

  async signIn(): Promise<void> {
    this.authStatus = '';
    const err = await this.auth.signIn(this.authEmail, this.authPassword);
    if (err) {
      this.authStatus = err;
      return;
    }
    await this.refreshAuth();
    this.authStatus = 'Signed in.';
  }

  async signOut(): Promise<void> {
    await this.auth.signOut();
    this.email = null;
    this.authStatus = 'Signed out.';
  }

  async loadComments(page: string): Promise<void> {
    try {
      this.comments = await this.api.getComments(page);
    } catch (err: any) {
      this.status = err.message || 'Failed to load comments';
    }
  }

  async submitComment(): Promise<void> {
    if (!this.post) {
      return;
    }
    this.status = '';
    try {
      await this.api.postComment({
        page: this.post.slug,
        author: this.form.author || 'user',
        rating: this.form.rating || null,
        message: this.form.message,
        email: this.email || undefined,
        meta: { hp: this.form.website, ts: this.form.ts }
      });
      this.form.message = '';
      this.form.ts = Date.now();
      await this.loadComments(this.post.slug);
      this.status = 'Comment submitted for approval.';
    } catch (err: any) {
      this.status = err.message || 'Failed to submit comment';
    }
  }
}
