import { promisify } from 'util';
import childProcess from 'child_process';

const exec = promisify(childProcess.exec);

export async function runCommand(command: string): Promise<string> {
  const { stdout, stderr } = await exec(command);

  // Ignore warnings
  if (stderr && !stderr.toLowerCase().includes('warning')) {
    throw new Error(stderr);
  }

  return stdout;
}
