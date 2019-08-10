#!/bin/sh

set -ex

create-admin-user -u ${ADMIN_USER} -p ${ADMIN_PASSWORD}
create-toshi-host bookmarks.json

server
