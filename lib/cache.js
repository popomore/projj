'use strict';

const assert = require('assert');
const fs = require('mz/fs');
const readJSON = require('utility').readJSON;

module.exports = class Cache {
  constructor(options) {
    assert(options && options.cachePath, 'cachePath is required');
    this.cachePath = options.cachePath;
  }

  async get(key) {
    if (!this.cache) {
      if (await fs.exists(this.cachePath)) {
        this.cache = await readJSON(this.cachePath);
        await this.setRepo(this.cache);
      } else {
        this.cache = {};
        await this.dump();
      }
    }
    return key ? this.cache[key] : this.cache;
  }

  async getKeys() {
    const cache = await this.get();
    return Object.keys(cache).filter(key => key !== 'version');
  }

  async set(key, value) {
    if (!key) return;
    if (!this.cache) await this.get();

    this.cache[key] = value || {};
  }

  async remove(keys) {
    if (!keys) return;
    if (!Array.isArray(keys)) keys = [ keys ];
    keys.forEach(key => delete this.cache[key]);
  }

  async dump() {
    if (!this.cache) return;
    await fs.writeFile(this.cachePath, JSON.stringify(this.cache, null, 2));
  }

  async setRepo(cache) {
    const keys = await this.getKeys();
    for (const key of keys) {
      if (cache[key] && cache[key].repo) continue;
      const option = cache[key] = {};
      const s = key.split('/');
      option.repo = `git@${s[0]}:${s[1]}/${s[2]}.git`;
    }
    await this.dump();
  }

  async upgrade() {
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

};
