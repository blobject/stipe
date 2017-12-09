#!/bin/sh

cd $HOME/src/collar/db/
rsync *.md alocy.be:/srv/alocy.be/db/ -tuv
cd $HOME/src/collar/ignore/img/
rsync *g alocy.be:/srv/alocy.be/ignore/img/ -tuv
