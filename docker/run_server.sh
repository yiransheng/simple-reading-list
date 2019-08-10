#!/bin/sh

set -ex

ls -alh

/create-toshi-index bookmarks.json

/server
