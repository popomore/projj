'use strict';

const fs = require('mz/fs');
const path = require('path');
const BaseCommand = require('../base_command');

class SyncCommand extends BaseCommand {

  * _run() {
    const base = this.config.base;
    const cache = yield this.cache.get();
    this.logger.info('Syncing cache from directory %s', base);
    for (const key of Object.keys(cache)) {
      if (yield fs.exists(path.join(base, key))) continue;
      this.childLogger.info('Remove %s that don\'t exist', key);
      yield this.cache.remove(key);
    }
    yield this.cache.dump();
  }

  help() {
    return 'Sync data from directory';
  }
}

module.exports = SyncCommand;
