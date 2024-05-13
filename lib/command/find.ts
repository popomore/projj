import chalk from 'chalk';
import clipboardy from 'clipboardy';
import utils from '../utils';
import BaseCommand from '../base_command';

interface IChooseResult {
  key: string;
}

class FindCommand extends BaseCommand {

  async _run(cwd: string, [ repo ]: string[]): Promise<void> {
    if (!repo) {
      this.logger.error('Please specify the repo name:');
      this.childLogger.error(chalk.white('For example:'), chalk.green('projj find', chalk.yellow('example')));
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

  async choose(choices: string[]): Promise<IChooseResult> {
    return await this.prompt({
      name: 'key',
      type: 'list',
      message: 'Please select the correct repo',
      choices,
    });
  }

  async copyPath(repo: string, dir: string): Promise<void> {
    try {
      this.logger.info('find repo %s\'s location: %s', repo, dir);
      await clipboardy.write(`cd ${dir}`);
      this.logger.info(chalk.green('📋  Copied to clipboard') + ', just use Ctrl+V');
    } catch (e) {
      this.logger.warn('Fail to copy to clipboard, error: %s', e.message);
    }
  }

  get description(): string {
    return 'Find repository';
  }

}

export default FindCommand;
