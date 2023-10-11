#!/bin/bash -eu

(
  set -x
  redis-cli -h "${REDIS_DB:-localhost}" --pipe < db.rdb
) 2>&1

if [ -n "${HANG:-}" ]; then
  trap 'echo "SIGINT" && exit 0' SIGINT
  while true; do
    sleep 1
  done
fi
