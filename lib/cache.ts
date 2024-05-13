import * as assert from 'assert';
import * as fs from 'mz/fs';
import { readJSON } from 'utility';

interface CacheOptions {
  cachePath: string;
}

interface CacheContent {
  [key: string]: any;
  version?: string;
}

export default class Cache {
  private cachePath: string;
  private cache?: CacheContent;

  constructor(options: CacheOptions) {
    assert(options && options.cachePath, 'cachePath is required');
    this.cachePath = options.cachePath;
  }

  async get(key?: string): Promise<any> {
    if (!this.cache) {
      if (await fs.exists(this.cachePath)) {
        this.cache = await readJSON(this.cachePath) as CacheContent;
        await this.setRepo(this.cache);
      } else {
        this.cache = {};
        await this.dump();
      }
    }
    return key ? this.cache[key] : this.cache;
  }

  async getKeys(): Promise<string[]> {
    const cache = await this.get();
    return Object.keys(cache).filter(key => key !== 'version');
  }

  async set(key: string, value?: any): Promise<void> {
    if (!key) return;
    if (!this.cache) await this.get();

    this.cache[key] = value || {};
  }

  async remove(keys: string | string[]): Promise<void> {
    if (!keys) return;
    if (!Array.isArray(keys)) keys = [ keys ];
    keys.forEach(key => delete this.cache[key]);
  }

  async dump(): Promise<void> {
    if (!this.cache) return;
    await fs.writeFile(this.cachePath, JSON.stringify(this.cache, null, 2));
  }

  private async setRepo(cache: CacheContent): Promise<void> {
    const keys = await this.getKeys();
    for (const key of keys) {
      if (cache[key] && cache[key].repo) continue;
      const option = cache[key] = {};
      const s = key.split('/');
      option.repo = `git@${s[0]}:${s[1]}/${s[2]}.git`;
    }
    await this.dump();
  }

  async upgrade(): Promise<void> {
    const cache = await this.get();
    switch (cache.version) {
      // v1 don't upgrade
      case 'v1':
        /* istanbul ignore next */
        return;
      default:
    }

    cache.version = 'v1';

    await this.dump();
  }
}
