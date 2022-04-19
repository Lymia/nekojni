#!/bin/bash

rm -v nekojni_utils/src/moe/lymia/nekojni/*.class
javac -verbose --source 1.8 --target 1.8 nekojni_utils/src/moe/lymia/nekojni/*.java
