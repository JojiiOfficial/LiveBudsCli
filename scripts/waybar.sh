#!/bin/sh
BUDS_STATUS=`earbuds status -o json -q`

REQ_STATUS=`echo $BUDS_STATUS | jq '.status' -r`
if [ "$REQ_STATUS" != "success" ]; then
    #echo
	echo "{\"text\": \"﫽 -- : 﫽 --\", \"class\" : \"Buds+\", \"percentage\" : \"﫽 -- : 﫽 -- \"}"
    exit 0;
fi

LEFT=$(echo $BUDS_STATUS | jq -r '.payload.batt_left')
RIGHT=$(echo $BUDS_STATUS | jq -r '.payload.batt_right')

LS="`echo $BUDS_STATUS | jq '.payload.placement_left'`"
case $LS in
	"3")
		LEFT="⚡ $LEFT"
		;;
	"2")
		LEFT="🅰 $LEFT"
		;;
	*)
		LEFT = "﫽 $LEFT"
		;;
esac

RS="`echo $BUDS_STATUS | jq '.payload.placement_right'`"
case $RS in
	"3")
		RIGHT="⚡ $RIGHT"
		;;
	"2")
		RIGHT="💡 $RIGHT"
		;;
	*)
		RIGHT = "﫽 $RIGHT"
		;;
esac

echo "{\"text\":\"$LEFT : $RIGHT\", \"class\":\"Buds+\", \"percentage\":\"$LEFT : $RIGHT\"}"



