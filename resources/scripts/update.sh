#!/bin/sh

cd $HOME/src/collar/db
rsync *.md alocy.be:/srv/alocy.be/db -tuv
rsync *.md alocy.be:/srv/dev.alocy.be/db -tuv
