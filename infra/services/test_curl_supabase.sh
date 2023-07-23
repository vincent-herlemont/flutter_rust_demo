#!/usr/bin/env bash

ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0"
SERVICE_ROLE_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImV4cCI6MTk4MzgxMjk5Nn0.EGIM96RAZx35lJzdJsyH-qQwv8Hdp7fsn3W0YpN81IU"

SUPABASE_URL="http://localhost:54321"

#curl -X GET \
#     -H "apiKey: $ANON_KEY" \
#     -H "Authorization: Bearer $SERVICE_ROLE_KEY" \
#     -H "Content-Type: application/json" \
#      "${URL}/rest/v1/hub_info"

export SUPABASE_URL="http://supabase_kong_infra:54321"
export EMAIL="test@test.test"
export PASSWORD="test"

TOKEN=$(curl \
  --request POST \
  --header "Content-Type: application/json" \
  --data "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}" \
  "${SUPABASE_URL}/auth/v1/token?grant_type=password" | jq -r '.access_token')

# Check is token is valid
curl -vvv \
   --request GET \
   --header "Authorization: Bearer $TOKEN" \
    "${SUPABASE_URL}/auth/v1/user"
