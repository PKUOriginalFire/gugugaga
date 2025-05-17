export DISPLAY=:0
export XDG_RUNTIME_DIR=/run/user/$UID

echo "Starting LinuxQQ..."
bash -c "$HOME/QQ/AppRun & $HOME/gugugaga --ws-server=ws://danmaku-server:5099/danmaku & wait -n; kill 0"
