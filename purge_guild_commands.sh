#!/bin/sh

APPLICATION_ID=$(cat .env | rg -U "^APP_ID=(.*)$" -r '$1') 
GUILD_ID=$(cat .env | rg -U "^GUILD_ID=(.*)$" -r '$1') 
TOKEN=$(cat .env | rg -U "^TOKEN=(.*)$" -r '$1') 

COMMANDS=$(curl --url "https://discord.com/api/v10/applications/$APPLICATION_ID/guilds/$GUILD_ID/commands" \
  --header "Authorization: Bot $TOKEN")

echo $COMMANDS

IDS=$(echo $COMMANDS | jq '.[] | [ .id ] | .[0]' | sed 's/"//g')

for id in $IDS
do
  echo Deleting $id
  curl --request DELETE \
 --url "https://discord.com/api/v10/applications/$APPLICATION_ID/guilds/$GUILD_ID/commands/$id" \
  --header "Authorization: Bot $TOKEN"
done
