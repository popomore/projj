'use strict';

const path = require('path');
const fs = require('mz/fs');
const ora = require('ora');
const runscript = require('runscript');
const chalk = require('chalk');
const BaseCommand = require('../base_command');

class ImportCommand extends BaseCommand {
  * _run(cwd, [ from ]) {
    this.count = 0;
    this.spinner = ora('Searching ' + from).start();
    const repos = yield this.findDirs(from);
    this.spinner.stop();

    for (const repo of repos) {
      const key = this.url2dir(repo);
      const targetPath = path.join(this.config.base, key);
      this.logger.info('Start importing repository %s', chalk.green(repo));
      if (yield fs.exists(targetPath)) {
        this.logger.warn(chalk.yellow('%s exists'), targetPath);
        continue;
      }
      // git clone
      try {
        yield this.gitClone(repo, key);
      } catch (_) {
        this.error(`Fail to clone ${repo}`);
      }
    }
  }

  * findDirs(cwd) {
    this.spinner.text = `Found ${chalk.cyan(this.count)}, Searching ${cwd}`;
    const dirs = yield fs.readdir(cwd);

    // match the directory
    if (dirs.includes('.git')) {
      try {
        const { stdout } = yield runscript('git config --get remote.origin.url', { stdio: 'pipe', cwd });
        this.spinner.text = `Found ${chalk.cyan(this.count++)}, Searching ${cwd}`;
        return [ stdout.toString().slice(0, -1) ];
      } catch (e) {
        // it contains .git, but no remote.url
        return [];
      }
    }

    // ignore node_modules
    if (dirs.includes('node_modules')) {
      return [];
    }

    let gitdir = [];
    for (const dir of dirs) {
      const subdir = path.join(cwd, dir);
      const stat = yield fs.stat(subdir);
      if (!stat.isDirectory()) {
        continue;
      }
      const d = yield this.findDirs(subdir);
      gitdir = gitdir.concat(d);
    }
    return gitdir;
  }

  help() {
    return 'Import repositories from existing directory';
  }

}

module.exports = ImportCommand;
