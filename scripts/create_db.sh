#!/bin/bash
docker run --name actix_sqlx \
       -p 3306:3306 \
       -e MARIADB_ROOT_USER=user \
       -e MARIADB_ROOT_PASSWORD=password \
       -e MARIADB_DATABASE=actix_sqlx \
       -v $PWD/scripts/init.sql:/docker-entrypoint-initdb.d/init.sql \
       -d mariadb:latest
