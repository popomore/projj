import BaseCommand from '../base_command';

class InitCommand extends BaseCommand {
  async _run(): Promise<void> {
    console.log(this.config);
    this.logger.info('Set base directory: %s', this.config.base.join(','));
  }

  get description(): string {
    return 'Initialize configuration';
  }
}

export default InitCommand;
