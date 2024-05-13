import * as fs from 'mz/fs';
import BaseCommand from '../base_command';

class SyncCommand extends BaseCommand {

  async _run(): Promise<void> {
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

  get description(): string {
    return 'Sync data from directory';
  }
}

export default SyncCommand;
