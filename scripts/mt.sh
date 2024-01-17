
current_dir=$(pwd)

pids=$(pgrep mounttab)
for pid in $pids
do
    echo "Checking $pid"
    cwd=$(readlink /proc/$pid/cwd)
    echo "CWD: $cwd"
    echo "Current dir: $current_dir"
    if [ "$cwd" == "$current_dir" ]
    then
        echo "Killing $pid"
        kill -15 $pid
        sleep 1
        echo "Killing again $pid"
        kill -9 $pid
        exit 0
    fi
done

cargo run -- start &

