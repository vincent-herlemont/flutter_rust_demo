#!/bin/bash

ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0"

# Get current time in nanoseconds (Unix Epoch time)
current_time=$(date +%s%N)

# Prepare the data payload with the current time
data='{"streams": [{ "stream": { "toto": "vincent" }, "values": [ [ "'$current_time'", "fizzbuzz" ] ] }]}'

# Make the POST request to Loki
curl -v -H "Content-Type: application/json" \
     -H "Authorization: Bearer $ANON_KEY" \
     -X POST -s "http://localhost:3101/loki/api/v1/push" --data-raw "$data"
