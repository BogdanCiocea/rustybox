#!/bin/bash
# 10

outputfile=$1
testfile=$2
examplefile=$outputfile.example

rm -rf output/*

rustybox grep '^runner.*' /etc/passwd &> $outputfile
scriptresult=$?

if [ $scriptresult == 0 ]
then
    grep '^runner.*' /etc/passwd &> $examplefile
    diff -y --suppress-common-lines $examplefile $outputfile &> $testfile
    testresult=$?

    rm -rf $examplefile

    rm -rf output/*

    if [ $testresult != 0 ]
    then
        echo "Incorrect output."
        exit -1
    fi
else
    rm -rf output/*
    echo "Command does not return 0 ($scriptresult)." > $testfile
    exit -1
fi