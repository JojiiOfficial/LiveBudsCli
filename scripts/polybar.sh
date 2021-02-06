#!/usr/bin/env bash
BUDS_STATUS=`earbuds status -o json -q`

REQ_STATUS=`echo $BUDS_STATUS | jq '.status' -r`
if [ "$REQ_STATUS" == "error" ];
then
    echo
    exit 1;
fi

LEFT=$(echo $BUDS_STATUS | jq -r '.payload.batt_left')
RIGHT=$(echo $BUDS_STATUS | jq -r '.payload.batt_right')

if [ "`echo $BUDS_STATUS | jq '.payload.placement_left'`" == "3" ];
then
    LEFT="⚡$LEFT"
fi

if [ "`echo $BUDS_STATUS | jq '.payload.placement_right'`" == "3" ];
then
    RIGHT="⚡$RIGHT"
fi


echo "(L: $LEFT% | R: $RIGHT%)"
