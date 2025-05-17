#!/bin/bash

XUID=$(id -u xuser)
XGID=$(id -g xuser)
XHOME=/home/xuser

chown -R $XUID:$XGID $XHOME/.config

export XDG_RUNTIME_DIR=/run/user/$XUID

echo "Starting Xpra server..."
gosu xuser xpra start :0 --bind-tcp=0.0.0.0:14500 --no-daemon --start-child="bash $XHOME/app.sh" --exit-with-children
