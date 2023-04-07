# .trove

## .trove is a CLI tool used to manage dotfiles.


### Thoughts:

NAME | HOST_PATH | CATEGORIES

NAME is for folder management in the repository? do we need a name?
HOST_PATH is the path on the target.
CATEGORIES is an array of categories, strings

operations:
 - add [PATH] [NAME]: moves file to storage space, symlinks back to original path; saves in config using NAME and PATH
   - optional "save path" for stuff that is dynamic (i.e. $HOME instead of /home/user)
 - remove [PATH] : removes symlink, copies file back to original path
    - "d" option to also delete from repo
 - deploy [optional: CATEGORY,NAME] : symlinks all into paths, or just provided CATEGORY or NAME
 - pack [optional: CATEGORY,NAME]: remove all symlinks, or jsut provided CATEGORY or NAME
 - init: setup path for where files are moved (likely, a git repo). Config gets saved as `.trove.conf`
   - also sticks a `.trove` file in $HOME which shows where to find this config/repo
 - status: show list of what names are deployed (GREEN) and what isnt (RED)