'use strict';

const path = require('path');
const chalk = require('chalk');
const inquirer = require('inquirer');
const clipboardy = require('clipboardy');
const BaseCommand = require('../base_command');

class AddCommand extends BaseCommand {

  * _run(cwd, [ repo ]) {
    const keys = Object.keys(this.cache);
    let matched = keys.filter(key => key.endsWith(repo.replace(/^\/?/, '/')));
    if (!matched.length) matched = keys.filter(key => key.indexOf(repo) >= 0);

    if (!matched.length) {
      this.logger.error('Can not find repo %s', chalk.yellow(repo));
      return;
    }
    if (matched.length === 1) {
      yield this.copyPath(repo, matched[0]);
      return;
    }
    // multi
    const res = yield this.choose(matched);
    yield this.copyPath(repo, res.key);
  }

  * choose(choices) {
    return yield inquirer.prompt({
      name: 'key',
      type: 'list',
      message: 'Please select the correct repo',
      choices,
    });
  }

  * copyPath(repo, key) {
    const dir = path.join(this.config.base, key);
    try {
      this.logger.info('find repo %s\'s location: %s', repo, dir);
      yield clipboardy.write(`cd ${dir}`);
      this.logger.info(chalk.green('📋  Copied to clipboard') + ', just use Ctrl+V');
    } catch (e) {
      this.logger.warn('Fail to copy to clipboard, error: %s', e.message);
    }
  }

  help() {
    return 'Find repository';
  }

}

module.exports = AddCommand;
