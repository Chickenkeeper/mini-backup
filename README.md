# mini-backup

A super simple command line backup utility for Windows.

## Usage

mini-backup expects two arguments:

1. A path to a text file containing a list of directories to back up
2. A path to a folder where the backup will be placed

## Behaviour

- mini-backup uses robocopy under the hood, with hardcoded arguments. These arguments will cause it to copy all attributes of all subdirectories, including hidden files and file creation dates, but it will ignore symlinks.
- The backup will be contained in a folder named `Backup DD-MM-YYYY`, making the assumption that there will be no more than one backup per day.
- The directory structure of all backed up files will be preserved including the drive letter, e.g. `\Backup DD-MM-YYYY\C\Users\...`

## Important Notes

- All directories in the list of directories must be full paths starting at the drive letter. Newlines seperating chunks of directories are accepted.
- Any issue during the backup process will cause mini-backup to exit, which may leave incomplete files. The backup can be reattempted immediately and will automatically skip files that have already been copied, but incomplete copies may not be repaired so it's best to delete the previous attempt before trying again.
- The default progress output of robocopy is piped straight to the terminal, which can cause a lot of clutter.
- An administrator command line may be needed to copy some files and permissions.
