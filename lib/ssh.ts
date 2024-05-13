#!/usr/bin/env node

import { spawn } from 'child_process';

const args: string[] = [
  '-o', 'StrictHostKeyChecking=no',
].concat(process.argv.slice(2));
const opt = {
  stdio: 'inherit',
};

spawn('ssh', args, opt);
