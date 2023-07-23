#!/bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"


pushd "$DIR/../"
supabase stop && supabase start
popd