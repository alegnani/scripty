#!/bin/bash
for dir in ./*; do
    lang=$(echo $dir | sed 's/^..//')
    echo Found: $lang in dir: $dir
    docker build $dir -t $(echo $lang)_executor
done
