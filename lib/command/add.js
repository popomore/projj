'use strict';

const path = require('path');
const fs = require('mz/fs');
const chalk = require('chalk');
const clipboardy = require('clipboardy');
const BaseCommand = require('../base_command');

class AddCommand extends BaseCommand {

  * _run(cwd, [ repo ]) {
    const key = this.url2dir(repo);
    const targetPath = path.join(this.config.base, key);
    this.logger.info('Start adding repository %s', chalk.green(repo));

    if (yield fs.exists(targetPath)) {
      throw new Error(`${targetPath} already exist`);
    }

    // preadd hook
    yield this.runHook('preadd', key);
    // git clone
    yield this.gitClone(repo, key);
    // postad hook
    yield this.runHook('postadd', key);

    try {
      yield clipboardy.write(`cd ${targetPath}`);
      this.logger.info(chalk.green('📋  Copied to clipboard') + ', just use Ctrl+V');
    } catch (e) {
      this.logger.warn('Fail to copy to clipboard, error: %s', e.message);
    }
  }

  help() {
    return 'Add repository';
  }

}

module.exports = AddCommand;
