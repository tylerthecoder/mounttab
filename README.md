# Mount Tab

MountTab is a system that stores your open browser sessions as a file on your computer. This allows a user to group tabs by sessions and close the window and reopen it. 

Inspired by https://omar.website/tabfs/

## Install

Install (bun)[https://bun.sh/]

```
bun run build
```

Will install to ~/.local/bin

Load the `./pkg` dir as a chrome extension. Start the chrome extension in an existing browser session

## Usage

```
mt serve
```

Starts a web server that listens to the chrome extension


```
mt start <workspace>
```

Starts a new browser session with the tabs in the workspace. Create a new session if none exist.

```
mt list-workspaces
```
Lists all workspaces in the state file. 


## Philosophy

I believe that your entire session state should be saved to disk in an easy to read format. I find myself having tabs open for a project and switching projects a lot. Starting on an old project is hard when I have to start from scratch and open up all the tabs I car about.


## Next Features

- [ ] Add system service file
- [ ] Add UI to chrome extension to manually assign a window to a workspace. 
- [ ] Add check for starting workspace that already exists


