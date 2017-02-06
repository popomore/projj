#!/usr/bin/env node

'use strict';

const spawn = require('child_process').spawn;

const args = [
  '-o', 'StrictHostKeyChecking=no',
].concat(process.argv.slice(2));
const opt = {
  stdio: 'inherit',
};

spawn('ssh', args, opt);
