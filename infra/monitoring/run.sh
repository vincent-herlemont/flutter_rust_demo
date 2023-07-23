#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

pushd "$DIR";
docker compose down -v && \
docker compose --env-file "$DIR/../services/.env" up -d --wait
docker compose logs -f
popd;