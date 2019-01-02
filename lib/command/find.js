'use strict';

const path = require('path');
const chalk = require('chalk');
const inquirer = require('inquirer');
const clipboardy = require('clipboardy');
const utils = require('../utils');
const BaseCommand = require('../base_command');

class FindCommand extends BaseCommand {

  * _run(cwd, [ repo ]) {
    const keys = Object.keys(yield this.cache.get());
    if (!repo) {
      this.logger.error('Please specify the repo name:');
      this.childLogger.error(chalk.white('For example:'), chalk.green('projj find', chalk.yellow('example')));
      return;
    }
    let matched = keys.filter(key => key.endsWith(repo.replace(/^\/?/, '/')));
    if (!matched.length) matched = keys.filter(key => key.indexOf(repo) >= 0);

    if (!matched.length) {
      this.logger.error('Can not find repo %s', chalk.yellow(repo));
      return;
    }
    let key;
    if (matched.length === 1) {
      key = matched[0];
    } else {
      const res = yield this.choose(matched);
      key = res.key;
    }
    const dir = path.join(this.config.base, key);
    if (this.config.change_directory) {
      /* istanbul ignore next */
      if (process.platform === 'darwin') {
        const script = utils.generateAppleScript(dir);
        this.logger.info(`Change directory to ${dir}`);
        yield this.runScript(script);
        return;
      }
      this.logger.error('Change directory only supported in darwin');
    }
    yield this.copyPath(repo, dir);
  }

  * choose(choices) {
    return yield inquirer.prompt({
      name: 'key',
      type: 'list',
      message: 'Please select the correct repo',
      choices,
    });
  }

  * copyPath(repo, dir) {
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

module.exports = FindCommand;
