# Mount Tab

Mount Tab syncs your open browser tabs to a file system.

Now you can store and launch sessions to certain file systems. 

Inspired by https://omar.website/tabfs/

## Usage

`cargo build --release`

`cargo install --path .`

`mounttab start`

Opens a new browser session. Reads the contents of `mount-tab.json`. As new tabs are made, the file is updated.

## Philosophy

I find myself having tabs open for a project and switching projects a lot. Starting on an old project is harder when it takes longer to get back into it. I normally work in a directory and would like to keep all state about that project in that directory. 

Eventually, I want every program I have open for a task to have a serialized state, so I can jump back into them quickly

