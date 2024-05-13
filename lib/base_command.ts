import * as inquirer from 'inquirer';
import * as path from 'path';
import * as fs from 'mz/fs';
import * as mkdirp from 'mz-modules/mkdirp';
import BaseCommand from 'common-bin';
import ConsoleLogger from 'zlogger';
import * as chalk from 'chalk';
import * as runscript from 'runscript';
import * as through from 'through2';
import * as giturl from 'giturl';
import { readJSON } from 'utility';
import Cache from './cache';

interface Config {
  base: string[];
  hooks: { [key: string]: string };
  alias: { [key: string]: string };
}

const configDir = path.join(process.env.HOME as string, '.projj');
const configPath = path.join(configDir, 'config.json');
const cachePath = path.join(configDir, 'cache.json');
const consoleLogger = new ConsoleLogger({
  time: false,
});

const defaults: Config = {
  base: [`${process.env.HOME}/projj`],
  hooks: {},
  alias: {
    'github://': 'https://github.com/',
  },
};

class Command extends BaseCommand {
  protected logger: ConsoleLogger;
  protected childLogger: ConsoleLogger;
  protected cache: Cache;
  protected config: Config;
  private PROMPT: any;

  constructor(rawArgv: string[]) {
    super(rawArgv);
    this.logger = new ConsoleLogger({
      prefix: chalk.green('✔︎  '),
      time: false,
    });
    this.childLogger = new ConsoleLogger({
      prefix: '   ',
      time: false,
      stdout: colorStream(process.stdout),
      stderr: colorStream(process.stderr),
    });
    this.cache = new Cache({ cachePath });
  }

  async run({ cwd, rawArgv }: { cwd: string; rawArgv: string[] }): Promise<void> {
    try {
      await this.init();
      await this._run(cwd, rawArgv);
      consoleLogger.info('✨  Done');
    } catch (err) {
      this.error(err.message);
      process.exit(1);
    }
  }

  async init(): Promise<void> {
    await this.loadConfig();

    const cache = await this.cache.get();

    if (!cache.version) {
      this.logger.warn('Upgrade cache');
      const baseDir = await this.chooseBaseDirectory();
      const keys = await this.cache.getKeys();
      for (const key of keys) {
        if (path.isAbsolute(key)) continue;
        const value = cache[key];
        await this.cache.remove([key]);
        await this.cache.set(path.join(baseDir, key), value);
      }

      await this.cache.upgrade();
    }
  }

  async loadConfig(): Promise<void> {
    await mkdirp(configDir);
    const configExists = await fs.exists(configPath);
    let config: Config;
    if (configExists) {
      config = await readJSON(configPath) as Config;
      config = resolveConfig(config, defaults);
      if (config.base) {
        this.config = config;
        return;
      }
    }

    const question = {
      type: 'input',
      name: 'base',
      message: 'Set base directory:',
      default: defaults.base,
    };
    const { base } = await this.prompt([question]);
    this.config = resolveConfig({ base }, defaults);
    await fs.writeFile(configPath, JSON.stringify(this.config, null, 2));
  }

  async runHook(name: string, cacheKey: string): Promise<void> {
    if (!this.config.hooks[name]) return;
    const hook = this.config.hooks[name];
    const env = {
      PATH: `${configDir}/hooks:${process.env.PATH}`,
      PROJJ_HOOK_NAME: name,
    };
    if (this.config[name]) {
      env.PROJJ_HOOK_CONFIG = JSON.stringify(this.config[name]);
    }
    const opt = {
      env: Object.assign({}, process.env, env),
    };

    const cwd = cacheKey;
    if (cwd && (await fs.exists(cwd))) opt.cwd = cwd;

    this.logger.info('Run hook %s for %s', chalk.green(name), cacheKey);
    await this.runScript(hook, opt);
  }

  async prompt(questions: inquirer.QuestionCollection<any>): Promise<inquirer.Answers> {
    if (!this.PROMPT) {
      this.PROMPT = inquirer.createPromptModule();
      const promptMapping = this.PROMPT.prompts;
      for (const key of Object.keys(promptMapping)) {
        const Clz = promptMapping[key];
        promptMapping[key] = class CustomPrompt extends Clz {
          static get name() { return Clz.name; }
          run() {
            process.send && process.send({ type: 'prompt', name: this.opt.name });
            process.emit('message', { type: 'prompt', name: this.opt.name });
            return super.run();
          }
        };
      }
    }
    return this.PROMPT(questions);
  }

  async runScript(cmd: string, opt: any): Promise<void> {
    const stdout = through();
    stdout.pipe(this.childLogger.stdout, { end: false });
    opt = Object.assign({}, {
      stdio: 'pipe',
      stdout,
    }, opt);
    try {
      await runscript(cmd, opt);
    } catch (err) {
      const stderr = err.stdio.stderr;
      if (stderr) {
        this.childLogger.info(stderr.toString());
      }
      throw err;
    }
  }

  error(msg: string): void {
    consoleLogger.error(chalk.red('✘  ' + msg));
  }

  url2dir(url: string): string {
    url = giturl.parse(url);
    return url.replace(/https?:\/\//, '');
  }

  async addRepo(repo: string, cacheKey: string): Promise<void> {
    await this.runHook('preadd', cacheKey);

    const targetPath = cacheKey;
    this.logger.info('Cloning into %s', chalk.green(targetPath));

    const env = Object.assign({
      GIT_SSH: path.join(__dirname, 'ssh.js'),
    }, process.env);
    await this.runScript(`git clone ${repo} ${targetPath} > /dev/null`, {
      env,
    });
    await this.cache.set(cacheKey, { repo });
    await this.cache.dump();

    await this.runHook('postadd', cacheKey);
  }

  async chooseBaseDirectory(): Promise<string> {
    const baseDirectories = this.config.base;
    if (baseDirectories.length === 1) return baseDirectories[0];

    const question = {
      type: 'list',
      name: 'base',
      message: 'Choose base directory',
      choices: baseDirectories,
    };
    const { base } = await this.prompt([question]);
    return base;
  }
}

function resolveConfig(config: Partial<Config>, defaults: Config): Config {
  config = Object.assign({}, defaults, config);
  if (!Array.isArray(config.base)) {
    config.base = [config.base];
  }
  config.base = config.base.map(baseDir => {
    switch (baseDir[0]) {
      case '.':
        return path.join(path.dirname(configPath), baseDir);
      case '~':
        return baseDir.replace('~', process.env.HOME as string);
      case '/':
        return baseDir;
      default:
        return path.join(process.cwd(), baseDir);
    }
  });

  return config as Config;
}

function colorStream(stream: NodeJS.WritableStream): NodeJS.WritableStream {
  const s = through(function(buf, _, done) {
    done(null, chalk.gray(buf.toString()));
  });
  s.pipe(stream);
  return s;
}
