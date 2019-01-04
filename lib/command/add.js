'use strict';

const path = require('path');
const fs = require('mz/fs');
const chalk = require('chalk');
const clipboardy = require('clipboardy');
const utils = require('../utils');
const BaseCommand = require('../base_command');

class AddCommand extends BaseCommand {

  * _run(cwd, [ repo ]) {
    repo = this.normalizeRepo(repo);
    const key = this.url2dir(repo);
    const targetPath = path.join(this.config.base, key);
    this.logger.info('Start adding repository %s', chalk.green(repo));

    if (yield fs.exists(targetPath)) {
      throw new Error(`${targetPath} already exist`);
    }

    yield this.addRepo(repo, key);

    if (this.config.change_directory) {
      /* istanbul ignore next */
      if (process.platform === 'darwin') {
        const script = utils.generateAppleScript(targetPath);
        this.logger.info(`Change directory to ${targetPath}`);
        yield this.runScript(script);
        return;
      }
      this.logger.error('Change directory only supported in darwin');
    }

    try {
      yield clipboardy.write(`cd ${targetPath}`);
      this.logger.info(chalk.green('📋  Copied to clipboard') + ', just use Ctrl+V');
    } catch (e) {
      this.logger.warn('Fail to copy to clipboard, error: %s', e.message);
    }
  }

  normalizeRepo(repo) {
    const alias = this.config.alias;
    const keys = Object.keys(alias);
    for (const key of keys) {
      // github://popomore/projj -> git@github.com:popomore/projj.git
      if (repo.startsWith(key)) {
        repo = alias[key] + repo.substring(key.length) + '.git';
        break;
      }
    }
    return repo;
  }

  help() {
    return 'Add repository';
  }

}

module.exports = AddCommand;
