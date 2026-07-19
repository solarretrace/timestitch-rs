#!/usr/bin/sh
set -e
mkdir -p /usr/local/man/man1
mkdir -p /usr/local/man/man5
install -g 0 -o 0 -m 0644 timestitch.1 /usr/local/man/man1/awol.1
gzip /usr/local/man/man1/awol.1
install -g 0 -o 0 -m 0644 timestitch.5 /usr/local/man/man5/awol.5
gzip /usr/local/man/man5/awol.5

