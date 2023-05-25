# mini-backup

A super simple command line backup utility for Windows.

## Usage

mini-backup expects two arguments:

1. A path to a text file containing a list of paths to files and/or folders to back up
2. A path to a folder where the backup will be created

## Behaviour

- mini-backup uses robocopy under the hood, with hardcoded arguments. These arguments will cause it to copy subdirectories, hidden files, attributes, and timestamps, but symlinks will be ignored.
- The backup will be contained in a folder with a name in the format `Backup DD-MM-YYYY`, making the assumption that there will be no more than one backup per day.
- The directory structure of all backed up files will be preserved including the drive letter, e.g. `D:\Backup DD-MM-YYYY\C\Users\...`

## Important Notes

- Each path in the list of paths to back up may be absolute or relative. Newlines will be skipped, so can be used to visually separate groups of paths.
- Any issue during the backup process will cause mini-backup to exit, which may leave incomplete files. The backup can be reattempted immediately and will automatically skip files that have already been copied, but incomplete copies may not be repaired so it's best to delete the previous attempt before trying again.
- The default progress output of robocopy is piped straight to the terminal, which can cause a lot of clutter.
- Administrator privileges may be needed to copy some files and permissions.
