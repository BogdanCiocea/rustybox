#!/bin/bash
# 25

outputfile=$1
testfile=$2


rustybox ls -R -l &> $outputfile
scriptresult=$?

ls -R -l | tr -s ' ' | cut -d ' ' -f 1,3,4,5,7,8,9 | grep -v total > output/ls_out

node verify/ls/ls.js output/ls_out $outputfile > $testfile 2>> $outputfile
testresult=$?

rm -f output/ls_out

if [ $testresult == 0 ]
then
    if [ $scriptresult != 0 ]
    then
        echo "Correct ls does not return 0 exit code." > $testfile
        exit -1 
    fi
fi

exit $testresult


