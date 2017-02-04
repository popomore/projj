'use strict';

const inquirer = require('inquirer');
const path = require('path');
const fs = require('mz/fs');
const mkdirp = require('mkdirp');
const BaseCommand = require('common-bin').Command;
const ConsoleLogger = require('zlogger');
const cp = require('child_process');

const configDir = path.join(process.env.HOME, '.projj');
const configPath = path.join(configDir, 'config.json');
const cachePath = path.join(configDir, 'cache.json');

const defaults = {
  base: `${process.env.HOME}/projj`,
  hooks: {},
};

class Command extends BaseCommand {

  constructor() {
    super();
    this.logger = new ConsoleLogger({
      prefix: '> ',
      time: false,
    });
  }

  * run(cwd, args) {
    try {
      yield this.init();
      yield this._run(cwd, args);
    } catch (err) {
      this.logger.error(err.message);
      this.logger.error(err.stack);
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
    if (this.cache[key]) return;
    this.cache[key] = {};
    yield fs.writeFile(cachePath, JSON.stringify(this.cache, null, 2));
  }

  * runHook(name, cwd) {
    if (!this.config.hooks[name]) return;
    const hook = this.config.hooks[name];
    const env = {
      PATH: `${configDir}/hooks:${process.env.PATH}`,
      PROJJ_HOOK_NAME: name,
    };
    const opt = { env };
    if (cwd && (yield fs.exists(cwd))) opt.cwd = cwd;

    const args = hook.trim().split(/\s+/);
    const command = args.shift();
    yield this.spawn(command, args, opt);
  }

  spawn(command, args, options) {
    args = args || [];
    options = Object.assign({}, options, {
      stdio: 'pipe',
    });
    return new Promise(resolve => {
      resolve(cp.spawn(command, args, options));
    }).then(cp => this._handleChildProcess(cp));
  }

  fork(modulePath, args) {
    args = args || [];
    const options = {
      cwd: this.options.dest,
      stdio: [ 'pipe', 'pipe', 'pipe', 'ipc' ],
    };
    return new Promise(resolve => {
      resolve(cp.fork(modulePath, args, options));
    }).then(cp => this._handleChildProcess(cp));
  }

  _handleChildProcess(cp) {
    return new Promise((resolve, reject) => {
      this.logger.child(cp, '> ');
      cp.once('error', reject);
      cp.once('exit', (code, signal) => {
        if (code === 0) {
          resolve();
        } else {
          const err = new Error(`exit with code ${code}, signal ${signal}`);
          reject(err);
        }
      });
    });
  }
}

module.exports = Command;

function* readJSON(configPath) {
  const content = yield fs.readFile(configPath);
  return JSON.parse(content);
}

function resolveConfig(config, defaults) {
  if (defaults) {
    config = Object.assign({}, defaults, config);
  }
  if (config.base[0] === '.') {
    config.base = path.join(path.dirname(configPath), config.base);
  } else if (config.base[0] === '~') {
    config.base = config.base.replace('~', process.env.HOME);
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
