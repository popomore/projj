import BaseCommand from '../base_command';

interface RunCommandConfig {
  hooks: { [key: string]: string };
}

class RunCommand extends BaseCommand {
  config: RunCommandConfig;

  async _run(cwd: string, [hookName]: string[]): Promise<void> {
    if (!hookName || !this.config.hooks[hookName]) {
      throw new Error(`Hook "${hookName}" don't exist`);
    }

    await this.runHook(hookName, cwd);
  }

  get description(): string {
    return 'Run hook in current directory';
  }
}

export default RunCommand;
