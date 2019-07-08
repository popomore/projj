'use strict';

const path = require('path');
const chalk = require('chalk');
const FindCommand = require('./find');
const fs = require('mz/fs');
const rimraf = require('mz-modules/rimraf');

class RemoveCommand extends FindCommand {

  * _run(cwd, [ repo ]) {
    if (!repo) {
      this.logger.error('Please specify the repo name:');
      this.childLogger.error(chalk.white('For example:'), chalk.green('projj remove', chalk.yellow('example')));
      return;
    }
    const key = this.fideRepo(repo);
    if (!key) {
      return;
    }
    this.logger.info('Do you want to remove the repository', chalk.green(key));
    this.logger.info(chalk.red('Removed repository cannot be restored!'));
    const s = key.split('/');
    const res = yield this.confirm(`${s[1]}/${s[2]}`);
    if (res) {
      const dir = path.join(this.config.base, key);
      yield rimraf(dir);
      const parent = path.dirname(dir);
      if ((yield fs.readdir(parent)).length === 0) {
        yield rimraf(parent);
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


  help() {
    return 'Remove repository';
  }

}

module.exports = RemoveCommand;
