#!/bin/sh

src=$HOME/src/stipe
dst=agaric:/home/b/www/stipe

copy() {
  rsync -cv $src/$1/* $dst/$1/
}

copy db
