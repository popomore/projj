import * as path from 'path';
import * as chalk from 'chalk';
import BaseCommand from '../base_command';
import * as fs from 'mz/fs';
import * as rimraf from 'mz-modules/rimraf';
import { prompt } from 'inquirer';

interface IChooseResult {
  key: string;
}

class RemoveCommand extends BaseCommand {

  async _run(cwd: string, [repo]: string[]): Promise<void> {
    if (!repo) {
      this.logger.error('Please specify the repo name:');
      this.childLogger.error(chalk.white('For example:'), chalk.green('projj remove', chalk.yellow('example')));
      return;
    }
    const keys = await this.cache.getKeys();
    let matched = keys.filter(key => key.endsWith(repo.replace(/^\/?/, '/')));
    if (!matched.length) matched = keys.filter(key => key.indexOf(repo) >= 0);
    if (!matched.length) {
      this.logger.error('Can not find repo %s', chalk.yellow(repo));
      return;
    }
    let key: string;
    if (matched.length === 1) {
      key = matched[0];
    } else {
      const res: IChooseResult = await this.choose(matched);
      key = res.key;
    }
    this.logger.info('Do you want to remove the repository', chalk.green(key));
    this.logger.info(chalk.red('Removed repository cannot be restored!'));
    const s = key.split('/');
    const res = await this.confirm(`${s[s.length - 2]}/${s[s.length - 1]}`);
    if (res) {
      const dir = key;
      await rimraf(dir);
      const parent = path.dirname(dir);
      if ((await fs.readdir(parent)).length === 0) {
        await rimraf(parent);
      }
      await this.cache.remove(key);
      await this.cache.dump();
      this.logger.info('Successfully remove repository', chalk.green(key));
    } else {
      this.logger.info('Cancel remove repository', chalk.green(key));
    }
  }

  async confirm(repoName: string): Promise<boolean> {
    const res = await prompt({
      message: `Please type in the name of the repository to confirm. ${chalk.green(repoName)} \n`,
      name: 'userInput',
    });
    if (res.userInput === repoName) {
      return true;
    }
    const continueRes = await prompt({
      type: 'confirm',
      message: 'Do you want to continue?',
      name: 'continueToEnter',
      default: false,
    });
    if (continueRes.continueToEnter) {
      return await this.confirm(repoName);
    }
    return false;
  }

  async choose(choices: string[]): Promise<IChooseResult> {
    return await prompt({
      name: 'key',
      type: 'list',
      message: 'Please select the correct repo',
      choices,
    });
  }

  get description(): string {
    return 'Remove repository';
  }
}

export default RemoveCommand;
