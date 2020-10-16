#!/bin/sh

BUDS_STATUS=`earbuds status -o json`

REQ_STATUS=`echo $BUDS_STATUS | jq '.status' -r`
if [ "$REQ_STATUS" == "error" ];
then
    echo
    exit;
fi


OUT=$(echo $BUDS_STATUS | jq '("L: " + (.payload.batt_left|tostring) + "% | R: " + (.payload.batt_right|tostring) + "%")' -r)

echo "($OUT)"
