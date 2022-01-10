#!/bin/bash
SCRIPT_DIR=$( readlink -f $( dirname "${BASH_SOURCE[0]}" ))
cd $SCRIPT_DIR
source venv/bin/activate
python index.py $@
