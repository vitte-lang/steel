import { Injectable } from '@angular/core';
import { createClient, SupabaseClient } from '@supabase/supabase-js';

const ENV_KEY = 'steelEnv';

type EnvConfig = {
  SUPABASE_URL: string;
  SUPABASE_ANON_KEY: string;
};

function readEnv(): EnvConfig {
  const w = window as typeof window & { __steelEnv?: Partial<EnvConfig> };
  return {
    SUPABASE_URL: w.__steelEnv?.SUPABASE_URL || '',
    SUPABASE_ANON_KEY: w.__steelEnv?.SUPABASE_ANON_KEY || ''
  };
}

@Injectable({ providedIn: 'root' })
export class SupabaseService {
  private client: SupabaseClient;

  constructor() {
    const env = readEnv();
    this.client = createClient(env.SUPABASE_URL, env.SUPABASE_ANON_KEY, {
      auth: { persistSession: true }
    });
  }

  getClient(): SupabaseClient {
    return this.client;
  }

  async signIn(email: string, password: string): Promise<string | null> {
    const { data, error } = await this.client.auth.signInWithPassword({ email, password });
    if (error) {
      return error.message;
    }
    if (!data.session) {
      return 'No session created';
    }
    return null;
  }

  async signOut(): Promise<void> {
    await this.client.auth.signOut();
  }

  async getAccessToken(): Promise<string | null> {
    const { data } = await this.client.auth.getSession();
    return data.session?.access_token ?? null;
  }

  async getUserEmail(): Promise<string | null> {
    const { data } = await this.client.auth.getUser();
    return data.user?.email ?? null;
  }
}
