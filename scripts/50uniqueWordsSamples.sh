#!/bin/bash
#This script should be run from gutenberg-analysis

for i in `seq 1 50`;
do
    spark-submit --class App --master local[*] BookProject/target/scala-2.11/gutenberg-analysis-assembly-0.1.jar -t data/texts/ -o data/spark/ -s data/stopwords.txt -r 100
done  