# MountTab

A program that can mount the tabs that are open as a file system. Be able to save and relaunch sessions. Inspired by https://omar.website/tabfs/

Sessions are stored anywhere. You select the session that a Chrome window attaches to from the extension.

## Questions
- How do I distinguish between multiple browser sessions? 
    - You have to select the session from the browser.

## Goals
- Eventually I want to do this with tmux sessions and PDFs. I want a way to save my full OS session and store in a working directory. 

## Philosophy
Everything should be a file and a user of a program should be able to use the built-in Unix tools to manage data. Users should be able to build different frontends on top of the data.

Don't remake a way to do version control or storage. Let the user store the data where they want and be able to sync the files using the tools they want


## Tech Stuff

Step 1 is to start a session. Must do this in a directory with the mounttab command. `mounttab start <session-name>`

The session will become visible in the Chrome extension. You can connect it to it by selecting it in the window. 

There are two ways of connecting: opening all the tabs listed in the directory on the session in a new Chrome window, or overriding all the tabs with the currently open tabs of the Chrome window.




