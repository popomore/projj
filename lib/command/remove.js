'use strict';

const path = require('path');
const chalk = require('chalk');
const BaseCommand = require('../base_command');
const fs = require('mz/fs');
const rimraf = require('mz-modules/rimraf');

class RemoveCommand extends BaseCommand {

  * _run(cwd, [ repo ]) {
    const keys = Object.keys(yield this.cache.get());
    if (!repo) {
      this.logger.error('Please specify the repo name:');
      this.childLogger.error(chalk.white('For example:'), chalk.green('projj remove', chalk.yellow('example')));
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
    this.logger.info('Do you want to remove the repository', chalk.green(key));
    this.logger.info(chalk.red('Removed repository cannot be restored!'));
    const foo = key.split('/');
    const repoName = `${foo[foo.length - 2]}/${foo[foo.length - 1]}`;
    const res = yield this.confirm(repoName);
    if (res) {
      const dir = path.join(this.config.base, key);
      if (yield fs.exists(dir)) {
        yield rimraf(dir);
        const parent = path.dirname(dir);
        if (fs.readdirSync(parent).length === 0) {
          yield rimraf(parent);
          this.logger.info('Successfully remove empty folder', chalk.green(parent));
        }
      } else {
        this.logger.info(`remove ${key} that don't exist`);
      }
      yield this.cache.remove(key);
      yield this.cache.dump();
      this.logger.info('Successfully remove repository', chalk.green(key));
    } else {
      this.logger.info('Cancel remove repository ', chalk.green(key));
    }
  }


  * confirm(repoName) {
    const res = yield this.prompt({
      message: `Please type in the name of the repository to confirm. ${chalk.green(repoName)} \n`,
      name: 'userInput',
    });
    if (res.userInput === repoName) {
      return true;
    }
    const continueRes = yield this.prompt({
      type: 'confirm',
      message: 'Do you want to continue?',
      name: 'continueToEnter',
      default: false,
    });
    if (continueRes.continueToEnter) {
      return yield this.confirm(repoName);
    }
    return false;
  }


  * choose(choices) {
    return yield this.prompt({
      name: 'key',
      type: 'list',
      message: 'Please select the correct repo',
      choices,
    });
  }

  help() {
    return 'Remove repository';
  }

}

module.exports = RemoveCommand;
