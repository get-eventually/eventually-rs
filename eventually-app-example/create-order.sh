#!/bin/bash

ENDPOINT="http://localhost:8080/orders/$1"

curl -XPOST "$ENDPOINT/create" > /dev/null
curl -XPOST "$ENDPOINT/add-item" -d '{"item_sku":"TEST-SKU-1","quantity":1,"price":10.99}' > /dev/null
curl -XPOST "$ENDPOINT/add-item" -d '{"item_sku":"TEST-SKU-2","quantity":3,"price":2.49}' > /dev/null
curl -XPOST "$ENDPOINT/complete" > /dev/null
curl "$ENDPOINT" | jq
