'use strict';

const inquirer = require('inquirer');
const path = require('path');
const fs = require('mz/fs');
const ini = require('ini');
const mkdirp = require('mkdirp');
const BaseCommand = require('common-bin').Command;
const ConsoleLogger = require('zlogger');
const cp = require('child_process');

const configDir = path.join(process.env.HOME, '.projj');
const configPath = path.join(configDir, 'config');
const hookPath = path.join(configDir, 'hook');
const cachePath = path.join(configDir, 'cache.json');

const defaults = {
  base: `${process.env.HOME}/projj`,
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
      yield this._run(cwd, args);
    } catch (err) {
      this.logger.error(err.message);
      this.logger.error(err.stack);
      process.exit(1);
    }
  }

  * init() {
    yield this.loadConfig();
    yield this.loadHook();
    yield this.getCache();
  }

  * loadConfig() {
    yield mkdir(configDir);
    const configExists = yield fs.exists(configPath);
    let config;
    if (configExists) {
      config = yield readINI(configPath);
      config = resolveConfig(config);
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
    yield fs.writeFile(configPath, ini.stringify(this.config, { whitespace: true }));
  }

  * loadHook() {
    const hookExists = yield fs.exists(hookPath);
    this.hooks = {};
    if (hookExists) {
      this.hooks = yield readINI(hookPath);
    }
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
    if (!this.hooks[name]) return false;
    const hook = this.hooks[name];
    const env = {
      PATH: `${process.env.PATH}:${configDir}/commands`,
      PROJJ_HOOK_NAME: name,
    };
    const opt = { env };
    if (cwd && (yield fs.exists(cwd))) opt.cwd = cwd;

    const args = hook.trim().split(/\s+/);
    const command = args.shift();
    yield this.spawn(command, args, opt);
    return true;
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

function* readINI(configPath) {
  const content = yield fs.readFile(configPath, 'utf8');
  return ini.parse(content);
}

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
