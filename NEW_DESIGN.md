Ideating on how this should work. 

There should be a chrome extention that is always trying to connect via websockets to the server. Once connected it keeps sending the windows / tab group. 

The server will take this info and save it to a file. The server also converts the window ids to my "bench" sessions. 

Server has API {
    open(name: string): Opens a new tab group with the current name. 
}

When a session is opened, the server
- launches a new browser with all the windows opened 
    - use cli since chrome extention can't open a lot of windows quickly
- Waits for the chrome extention to report the new window. Assigns that window to the session name. 
- All tabs with that window id will be saved now. 

Server saves all tab info to single json file
"chrome_tabs.json"

I should just write this with typescript. I can do it so quickly in typescript. 

