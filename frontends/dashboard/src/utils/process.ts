import { promisify } from 'util';
import childProcess from 'child_process';

const exec = promisify(childProcess.exec);
const DEFAULT_TIMEOUT = 30000; // 30 seconds

export async function runCommand(command: string): Promise<string> {
  const { stdout, stderr } = await exec(command, { timeout: DEFAULT_TIMEOUT });

  // Ignore warnings
  if (stderr && !stderr.toLowerCase().includes('warning')) {
    throw new Error(stderr);
  }

  return stdout;
}
