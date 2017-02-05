'use strict';

const inquirer = require('inquirer');
const path = require('path');
const fs = require('mz/fs');
const mkdirp = require('mkdirp');
const BaseCommand = require('common-bin').Command;
const ConsoleLogger = require('zlogger');
const chalk = require('chalk');
const runscript = require('runscript');
const through = require('through2');
const giturl = require('giturl');

const configDir = path.join(process.env.HOME, '.projj');
const configPath = path.join(configDir, 'config.json');
const cachePath = path.join(configDir, 'cache.json');
const consoleLogger = new ConsoleLogger({
  time: false,
});

const defaults = {
  base: `${process.env.HOME}/projj`,
  hooks: {},
};

class Command extends BaseCommand {

  constructor() {
    super();
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
  }

  * run(cwd, args) {
    try {
      yield this.init();
      yield this._run(cwd, args);
      consoleLogger.info('✨  Done');
    } catch (err) {
      this.error(err.message);
      // this.logger.error(err.stack);
      process.exit(1);
    }
  }

  * init() {
    yield this.loadConfig();
    yield this.getCache();
  }

  * loadConfig() {
    yield mkdir(configDir);
    const configExists = yield fs.exists(configPath);
    let config;
    if (configExists) {
      config = yield readJSON(configPath);
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
    const { base } = yield inquirer.prompt([ question ]);
    this.config = resolveConfig({ base }, defaults);
    yield fs.writeFile(configPath, JSON.stringify(this.config, null, 2));
  }

  * getCache() {
    const exists = yield fs.exists(cachePath);
    this.cache = {};
    if (exists) {
      this.cache = yield readJSON(cachePath);
    }
  }

  * setCache(key) {
    if (!this.cache[key]) {
      this.cache[key] = {};
    }
    yield fs.writeFile(cachePath, JSON.stringify(this.cache, null, 2));
  }

  * runHook(name, cacheKey) {
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

    let cwd;
    if (this.cache[cacheKey]) {
      cwd = path.join(this.config.base, cacheKey);
    } else {
      cwd = cacheKey;
    }
    if (cwd && (yield fs.exists(cwd))) opt.cwd = cwd;

    this.logger.info('Run hook %s for %s', chalk.green(name), cacheKey);
    yield this.runScript(hook, opt);
  }

  * runScript(cmd, opt) {
    const stdout = through();
    stdout.pipe(this.childLogger.stdout, { end: false });
    const stderr = through();
    stderr.pipe(this.childLogger.stderr, { end: false });
    opt = Object.assign({}, opt, {
      stdio: 'pipe',
      stdout,
      stderr,
    });
    yield runscript(cmd, opt);
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
}

module.exports = Command;

function* readJSON(configPath) {
  const content = yield fs.readFile(configPath);
  return JSON.parse(content);
}

function resolveConfig(config, defaults) {
  config = Object.assign({}, defaults, config);
  switch (config.base[0]) {
    case '.':
      config.base = path.join(path.dirname(configPath), config.base);
      break;
    case '~':
      config.base = config.base.replace('~', process.env.HOME);
      break;
    case '/':
      break;
    default:
      config.base = path.join(process.cwd(), config.base);
  }
  return config;
}

function mkdir(file) {
  return new Promise((resolve, reject) => {
    mkdirp(file, err => {
      err ? reject(err) : resolve();
    });
  });
}

function colorStream(stream) {
  const s = through(function(buf, _, done) {
    done(null, chalk.gray(buf.toString()));
  });
  s.pipe(stream);
  return s;
}
