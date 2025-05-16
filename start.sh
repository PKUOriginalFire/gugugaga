#!/bin/bash
mkdir -p $HOME/.xpra
mkdir -p $XDG_RUNTIME_DIR
chmod 700 $XDG_RUNTIME_DIR

export DISPLAY=:0

xpra start :0 --bind-tcp=0.0.0.0:14500 --no-daemon \
    --start-child="bash -c \"( $HOME/QQ/AppRun & $HOME/gugugaga --ws-server=ws://danmaku-server:5099/danmaku & wait -n; kill 0 ) 2>/dev/null\"" \
    --exit-with-child=yes
