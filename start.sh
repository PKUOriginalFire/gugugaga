#!/bin/bash

XUID=$(id -u xuser)
XGID=$(id -g xuser)
XHOME=/home/xuser

chown -R $XUID:$XGID $XHOME/.config

export XDG_RUNTIME_DIR=/run/user/$XUID
export DISPLAY=:0

gosu xuser xpra start :0 --bind-tcp=0.0.0.0:14500 --no-daemon \
    --start-child="bash -c \"( $XHOME/QQ/AppRun & $XHOME/gugugaga --ws-server=ws://danmaku-server:5099/danmaku & wait -n; kill 0 ) 2>/dev/null\"" \
    --exit-with-child=yes
