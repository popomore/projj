'use strict';

const chalk = require('chalk');
const clipboardy = require('clipboardy');
const utils = require('../utils');
const BaseCommand = require('../base_command');

class ListCommand extends BaseCommand {

  async _run(cwd, [ repo = '' ]) {
    const keys = await this.cache.getKeys();

    if (!keys.length) {
      this.logger.error('Workspace is empty.');
      return;
    }

    let matched = keys;
    if (repo) matched = keys.filter(key => key.indexOf(repo) >= 0);
    const res = await this.choose(matched);
    const key = res.key;
    const dir = key;
    if (this.config.change_directory) {
      /* istanbul ignore next */
      if (process.platform === 'darwin') {
        const script = utils.generateAppleScript(dir);
        this.logger.info(`Change directory to ${dir}`);
        await this.runScript(script);
        return;
      }
      this.logger.error('Change directory only supported in darwin');
    }
    await this.copyPath(repo, dir);
  }

  async choose(choices) {
    const cache = await this.cache.get();
    const list = choices.map(key => {
      let name = key;
      if (this.config.base.length === 1) {
        name = key.replace(this.config.base[0], '')
          .replace(/^\/+/, '') +
          (cache[key]?.desc ?  ': ' + cache[key].desc : '');
      }
      return {
        name,
        value: key,
      };
    });
    return await this.prompt({
      name: 'key',
      type: 'rawlist',
      message: 'Please select one of the repo',
      choices: list,
    });
  }

  async copyPath(repo, dir) {
    try {
      this.logger.info('find repo %s\'s location: %s', repo, dir);
      await clipboardy.write(`cd ${dir}`);
      this.logger.info(chalk.green('📋  Copied to clipboard') + ', just use Ctrl+V');
    } catch (e) {
      this.logger.warn('Fail to copy to clipboard, error: %s', e.message);
    }
  }

  get description() {
    return 'List all of repositories';
  }

}

module.exports = ListCommand;
