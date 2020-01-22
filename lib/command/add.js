'use strict';

const path = require('path');
const fs = require('mz/fs');
const chalk = require('chalk');
const clipboardy = require('clipboardy');
const utils = require('../utils');
const BaseCommand = require('../base_command');

class AddCommand extends BaseCommand {

  async _run(_, [ repo ]) {
    repo = this.normalizeRepo(repo);
    const key = this.url2dir(repo);
    const base = await this.chooseBaseDirectory();
    const targetPath = path.join(base, key);
    this.logger.info('Start adding repository %s', chalk.green(repo));

    if (await fs.exists(targetPath)) {
      throw new Error(`${targetPath} already exist`);
    }

    await this.addRepo(repo, targetPath);

    if (this.config.change_directory) {
      /* istanbul ignore next */
      if (process.platform === 'darwin') {
        const script = utils.generateAppleScript(targetPath);
        this.logger.info(`Change directory to ${targetPath}`);
        await this.runScript(script);
        return;
      }
      this.logger.error('Change directory only supported in darwin');
    }

    try {
      await clipboardy.write(`cd ${targetPath}`);
      this.logger.info(chalk.green('📋  Copied to clipboard') + ', just use Ctrl+V');
    } catch (e) {
      this.logger.warn('Fail to copy to clipboard, error: %s', e.message);
    }
  }

  normalizeRepo(repo) {
    const alias = this.config.alias;
    const keys = Object.keys(alias);
    for (const key of keys) {
      // github://popomore/projj -> https://github.com/popomore/projj.git
      if (repo.startsWith(key)) {
        repo = alias[key] + repo.substring(key.length) + '.git';
        break;
      }
    }
    return repo;
  }

  get description() {
    return 'Add repository';
  }

}

module.exports = AddCommand;
