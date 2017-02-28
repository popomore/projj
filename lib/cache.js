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
      } else {
        this.cache = {};
        yield this.dump();
      }
    }
    return key ? this.cache[key] : this.cache;
  }

  * set(key) {
    if (!key) return;
    if (!this.cache) yield this.get();

    this.cache[key] = {};
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
};


function* readJSON(configPath) {
  const content = yield fs.readFile(configPath);
  return JSON.parse(content);
}
