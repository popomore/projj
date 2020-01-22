'use strict';

const fs = require('mz/fs');
const BaseCommand = require('../base_command');

class SyncCommand extends BaseCommand {

  * _run() {
    const base = this.config.base;
    this.logger.info('Syncing cache from directory %s', base);
    const keys = yield this.cache.getKeys();
    for (const key of keys) {
      if (yield fs.exists(key)) continue;
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
