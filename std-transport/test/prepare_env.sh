#!/bin/sh

python -m venv env
source env/bin/activate

pip install wheel
pip install pyserial
pip install erpc_module/erpc_python

