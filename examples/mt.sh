workspace=$(mt list-workspaces | rofi -dmenu -i -p "Select workspace")

if [ -z "$workspace" ]; then
    echo "No workspace selected"
    exit 1
fi

echo "Selected workspace: $workspace"

mt start $workspace




