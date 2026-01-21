import { CommonModule } from '@angular/common';
import { Component, OnInit } from '@angular/core';
import { ActivatedRoute } from '@angular/router';

import { AppState } from './app.state';

@Component({
  selector: 'app-blog-detail',
  imports: [CommonModule],
  templateUrl: './blog-detail.component.html'
})
export class BlogDetailComponent implements OnInit {
  post: any;

  constructor(private route: ActivatedRoute, private state: AppState) {}

  get t() {
    return this.state.t();
  }

  async ngOnInit(): Promise<void> {
    const slug = this.route.snapshot.paramMap.get('slug');
    this.post = this.t.blog.posts.find((p: any) => p.slug === slug);
  }
}
