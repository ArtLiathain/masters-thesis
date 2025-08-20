#! /bin/bash

set -e

xelatex thesis.tex || exit 1

bibtex thesis || exit 1
makeindex thesis.nlo -s nomencl.ist -o thesis.nls || exit 1

xelatex thesis.tex || exit 1
xelatex thesis.tex || exit 1
