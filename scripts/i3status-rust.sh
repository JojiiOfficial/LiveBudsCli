#!/bin/bash
BUDS_STATUS=`earbuds status -o json -q`

REQ_STATUS=`echo $BUDS_STATUS | jq '.status' -r`
if [ "$REQ_STATUS" != "success" ]; then
	echo "{\"text\":\"\"}"
    exit 1;
fi

LEFT=$(echo $BUDS_STATUS | jq -r '.payload.batt_left')
LS=$(echo $BUDS_STATUS | jq '.payload.placement_left')

RIGHT=$(echo $BUDS_STATUS | jq -r '.payload.batt_right')
RS=$(echo $BUDS_STATUS | jq '.payload.placement_right')

CASE=$(echo $BUDS_STATUS | jq '.payload.batt_case')

MINIMUM=101

if [ "$LEFT" -ne "0" ]
then
	MINIMUM=$(($MINIMUM>$LEFT ? $LEFT : $MINIMUM))
	case $LS in
		1)
			LEFT="🦻 $LEFT"
			;;
		2)
			LEFT="💡 $LEFT"
			;;
		3)
			LEFT="⚡ $LEFT"
			;;
		*)
			LEFT="﫽 $LEFT"
			;;
	esac
else
	LEFT=""
fi

if [ "$RIGHT" -ne "0" ]
then
	MINIMUM=$(($MINIMUM>$RIGHT ? $RIGHT : $MINIMUM))
	case $RS in
		1)
			RIGHT="🦻 $RIGHT"
			;;
		2)
			RIGHT="💡 $RIGHT"
			;;
		3)
			RIGHT="⚡ $RIGHT"
			;;
		*)
			RIGHT="﫽 $RIGHT"
			;;
	esac
else
	RIGHT=""
fi

if (($CASE <= 100))
then
	CASE=" 📦 $CASE"
else
	CASE=""
fi


case $MINIMUM in
	100)
		STATUS="Good"
		;;
	[5-9]*)
		STATUS="Good"
		;;
	[3-4]*)
		STATUS="Warning"
		;;
	[1-2]*)
		STATUS="Critical"
		;;
	[0-9])
		STATUS="Critical"
		;;
	*)
		STATUS="Info"
		;;
esac

echo "{\"icon\":\"\", \"state\":\"$STATUS\", \"text\":\"($LEFT : $RIGHT)$CASE\"}"


