'use strict';

const assert = require('assert');
const fs = require('mz/fs');


module.exports = class Cache {
  constructor(options) {
    assert(options && options.cachePath, 'cachePath is required');
    this.cachePath = options.cachePath;
  }

  * get(key) {
    if (!this.cache) {
      if (yield fs.exists(this.cachePath)) {
        this.cache = yield readJSON(this.cachePath);
        yield this.setRepo(this.cache);
      } else {
        this.cache = {};
        yield this.dump();
      }
    }
    return key ? this.cache[key] : this.cache;
  }

  * getKeys() {
    const cache = yield this.get();
    return Object.keys(cache).filter(key => key !== 'version');
  }

  * set(key, value) {
    if (!key) return;
    if (!this.cache) yield this.get();

    this.cache[key] = value || {};
  }

  * remove(keys) {
    if (!keys) return;
    if (!Array.isArray(keys)) keys = [ keys ];
    keys.forEach(key => delete this.cache[key]);
  }

  * dump() {
    if (!this.cache) return;
    yield fs.writeFile(this.cachePath, JSON.stringify(this.cache, null, 2));
  }

  * setRepo(cache) {
    const keys = yield this.getKeys();
    for (const key of keys) {
      if (cache[key] && cache[key].repo) continue;
      const option = cache[key] = {};
      const s = key.split('/');
      option.repo = `git@${s[0]}:${s[1]}/${s[2]}.git`;
    }
    yield this.dump();
  }

  * upgrade() {
    const cache = yield this.get();
    switch (cache.version) {
      // v1 don't upgrade
      case 'v1':
        /* istanbul ignore next */
        return;
      default:
    }

    cache.version = 'v1';

    yield this.dump();
  }

};


function* readJSON(configPath) {
  const content = yield fs.readFile(configPath);
  return JSON.parse(content);
}
