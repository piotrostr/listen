#!/bin/bash

pip install -U base58

cat "$1" | python3 -c "import sys, base58; print(base58.b58encode(bytes(eval(sys.stdin.read()))).decode())"
