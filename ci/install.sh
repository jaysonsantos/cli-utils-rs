#!/usr/bin/env bash
set -ex
set -o pipefail

BINS="delete-line delete-local-branches aws-flow-logs aws-ssm-env-importer aws-ssm-env-exporter fix-ksql-deleted-topics"

for bin in $BINS; do
  cargo install --path="$bin" --root=installed
done
