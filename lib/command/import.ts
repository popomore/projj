import * as path from 'path';
import * as fs from 'mz/fs';
import ora from 'ora';
import runscript from 'runscript';
import chalk from 'chalk';
import BaseCommand from '../base_command';

interface RepoOption {
  repo?: string;
}

class ImportCommand extends BaseCommand {
  private count: number;
  private spinner: ora.Ora;

  async _run(cwd: string, [from]: string[]): Promise<void> {
    let repos: string[] = [];
    if (from === '--cache') {
      const keys = await this.cache.getKeys();
      for (const key of keys) {
        const option: RepoOption = await this.cache.get(key);
        if (option.repo) repos.push(option.repo);
      }
      await this.cache.dump();
    } else {
      this.count = 0;
      this.spinner = ora('Searching ' + from).start();
      repos = await this.findDirs(from);
      this.spinner.stop();
    }

    const baseDir = await this.chooseBaseDirectory();
    for (const repo of repos) {
      const key = this.url2dir(repo);
      const targetPath = path.join(baseDir, key);
      this.logger.info('Start importing repository %s', chalk.green(repo));
      if (await fs.exists(targetPath)) {
        this.logger.warn(chalk.yellow('%s exists'), targetPath);
        continue;
      }
      try {
        await this.addRepo(repo, targetPath);
      } catch (_) {
        this.error(`Fail to clone ${repo}`);
      }
    }
  }

  async findDirs(cwd: string): Promise<string[]> {
    this.spinner.text = `Found ${chalk.cyan(this.count)}, Searching ${cwd}`;
    const dirs = await fs.readdir(cwd);

    // match the directory
    if (dirs.includes('.git')) {
      try {
        const { stdout } = await runscript('git config --get remote.origin.url', { stdio: 'pipe', cwd });
        this.spinner.text = `Found ${chalk.cyan(this.count++)}, Searching ${cwd}`;
        return [stdout.toString().slice(0, -1)];
      } catch (e) {
        // it contains .git, but no remote.url
        return [];
      }
    }

    // ignore node_modules
    if (dirs.includes('node_modules')) {
      return [];
    }

    let gitdir: string[] = [];
    for (const dir of dirs) {
      const subdir = path.join(cwd, dir);
      const stat = await fs.stat(subdir);
      if (!stat.isDirectory()) {
        continue;
      }
      const d = await this.findDirs(subdir);
      gitdir = gitdir.concat(d);
    }
    return gitdir;
  }

  get description(): string {
    return 'Import repositories from existing directory';
  }
}

export default ImportCommand;
