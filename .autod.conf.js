'use strict';

module.exports = {
  write: true,
  prefix: '^',
  test: [
    'test',
  ],
  dep: [
  ],
  devdep: [
    'egg-ci',
    'egg-bin',
    'autod',
    'eslint',
    'eslint-config-egg',
  ],
  exclude: [
    './test/fixtures',
    './dist',
  ],
  registry: 'https://r.cnpmjs.org',
};
