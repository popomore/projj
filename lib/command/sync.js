'use strict';

const fs = require('mz/fs');
const BaseCommand = require('../base_command');

class SyncCommand extends BaseCommand {

  async _run() {
    const base = this.config.base;
    this.logger.info('Syncing cache from directory %s', base);
    const keys = await this.cache.getKeys();
    for (const key of keys) {
      if (await fs.exists(key)) continue;
      this.childLogger.info('Remove %s that don\'t exist', key);
      await this.cache.remove(key);
    }
    await this.cache.dump();
  }

  get description() {
    return 'Sync data from directory';
  }
}

module.exports = SyncCommand;
