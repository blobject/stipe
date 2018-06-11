#!/bin/sh

src=$HOME/src/stipe
dst=agaric:/srv/agaric.net

copy() {
  rsync -cv $src/$1/* $dst/$1/
}

copy db
