# Mount Tab

Mount Tab syncs your open browser tabs to a file system.

Now you can store and launch sessions to certain file systems. 

Inspired by https://omar.website/tabfs/

## Usage

`mounttab start`

Opens a new browser session. Reads the contents of `mount-tab.json`. As new tabs are made, the file is updated.

## Development
`bun build.ts`

## Philosophy

I find myself having tabs open for a project and switching projects a lot. Starting on an old project is harder when it takes longer to get back into it. 

Everything should be a file and a user of a program should be able to use the built-in Unix tools to manage data. Users should be able to build different frontends on top of the data.

Don't remake a way to do version control or storage. Let the user store the data where they want and be able to sync the files using the tools they want

## To-do

Make a JSON option to store the data. 

Eventually I want to do this with tmux sessions and PDFs. I want a way to save my full OS session and store in a working directory. 
