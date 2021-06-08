'use strict';

const inquirer = require('inquirer');
const path = require('path');
const fs = require('mz/fs');
const mkdirp = require('mz-modules/mkdirp');
const BaseCommand = require('common-bin');
const ConsoleLogger = require('zlogger');
const chalk = require('chalk');
const runscript = require('runscript');
const through = require('through2');
const giturl = require('giturl');
const readJSON = require('utility').readJSON;
const Cache = require('./cache');
const PROMPT = Symbol('prompt');

const userHomeDir = process.platform === 'win32' ? process.env.USERPROFILE : process.env.HOME;
const configDir = path.join(userHomeDir, '.projj');
const configPath = path.join(configDir, 'config.json');
const cachePath = path.join(configDir, 'cache.json');
const consoleLogger = new ConsoleLogger({
  time: false,
});

const defaults = {
  base: `${userHomeDir}/projj`,
  hooks: {},
  alias: {
    'github://': 'https://github.com/',
  },
};

class Command extends BaseCommand {

  constructor(rawArgv) {
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

  async run({ cwd, rawArgv }) {
    try {
      await this.init();
      await this._run(cwd, rawArgv);
      consoleLogger.info('✨  Done');
    } catch (err) {
      this.error(err.message);
      // this.logger.error(err.stack);
      process.exit(1);
    }
  }

  async init() {
    await this.loadConfig();

    const cache = await this.cache.get();

    if (!cache.version) {
      this.logger.warn('Upgrade cache');
      const baseDir = await this.chooseBaseDirectory();
      const keys = await this.cache.getKeys();
      for (const key of keys) {
        if (path.isAbsolute(key)) continue;
        const value = cache[key];
        await this.cache.remove([ key ]);
        await this.cache.set(path.join(baseDir, key), value);
      }

      await this.cache.upgrade();
    }
  }

  async loadConfig() {
    await mkdirp(configDir);
    const configExists = await fs.exists(configPath);
    let config;
    if (configExists) {
      config = await readJSON(configPath);
      config = resolveConfig(config, defaults);
      // ignore when base has been defined in ~/.projj/config
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
    const { base } = await this.prompt([ question ]);
    this.config = resolveConfig({ base }, defaults);
    await fs.writeFile(configPath, JSON.stringify(this.config, null, 2));
  }

  async runHook(name, cacheKey) {
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

  async prompt(questions) {
    if (!this[PROMPT]) {
      // create a self contained inquirer module.
      this[PROMPT] = inquirer.createPromptModule();
      const promptMapping = this[PROMPT].prompts;
      for (const key of Object.keys(promptMapping)) {
        const Clz = promptMapping[key];
        // extend origin prompt instance to emit event
        promptMapping[key] = class CustomPrompt extends Clz {
          /* istanbul ignore next */
          static get name() { return Clz.name; }
          run() {
            process.send && process.send({ type: 'prompt', name: this.opt.name });
            process.emit('message', { type: 'prompt', name: this.opt.name });
            return super.run();
          }
        };
      }
    }
    return this[PROMPT](questions);
  }

  async runScript(cmd, opt) {
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

  error(msg) {
    consoleLogger.error(chalk.red('✘  ' + msg));
  }

  // https://github.com/popomore/projj.git
  // => $BASE/github.com/popomore/projj
  url2dir(url) {
    url = giturl.parse(url);
    return url.replace(/https?:\/\//, '');
  }

  async addRepo(repo, cacheKey) {
    // preadd hook
    await this.runHook('preadd', cacheKey);

    const targetPath = cacheKey;
    this.logger.info('Cloning into %s', chalk.green(targetPath));

    const env = Object.assign({
      GIT_SSH: path.join(__dirname, 'ssh.js'),
    }, process.env);
    await this.runScript(`git clone ${repo} ${targetPath} > ${process.platform === 'win32' ? 'NUL' : '/dev/null'}`, {
      env,
    });
    // add this repository to cache.json
    await this.cache.set(cacheKey, { repo });
    await this.cache.dump();

    // preadd hook
    await this.runHook('postadd', cacheKey);
  }

  async chooseBaseDirectory() {
    const baseDirectories = this.config.base;
    if (baseDirectories.length === 1) return baseDirectories[0];

    const question = {
      type: 'list',
      name: 'base',
      message: 'Choose base directory',
      choices: baseDirectories,
    };
    const { base } = await this.prompt([ question ]);
    return base;
  }
}

module.exports = Command;

function resolveConfig(config, defaults) {
  config = Object.assign({}, defaults, config);
  if (!Array.isArray(config.base)) {
    config.base = [ config.base ];
  }
  config.base = config.base.map(baseDir => {
    switch (baseDir[0]) {
      case '.':
        return path.join(path.dirname(configPath), baseDir);
      case '~':
        return baseDir.replace('~', process.env.HOME);
      case '/':
        return baseDir;
      default:
        if (process.platform === 'win32' && /^[A-Z]:/.test(baseDir)) {
          return path.join(baseDir);
        }
        return path.join(process.cwd(), baseDir);
    }
  });

  return config;
}

function colorStream(stream) {
  const s = through(function(buf, _, done) {
    done(null, chalk.gray(buf.toString()));
  });
  s.pipe(stream);
  return s;
}
