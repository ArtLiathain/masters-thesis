# UL Thesis Template

Based on [original UL thesis template](https://www.overleaf.com/latex/templates/university-of-limerick-thesis-template/dhhwcbhtjbcc)

# Changes and Additions:
- Citation style changed to IEEE
- Some changed to PDF generation flags to use xelatex
- Commitizen for semantic versioning
- Github Action to build document, save version

# Tooling
- Semantic versioning: Commitizen
- LSP: texlab
- Viewer: sumatra PDF
- Texlive: xelatex (to build pdf) and publishers (for IEEE style)

`sudo apt install texlive-xetex texlive-publishers`

# How to use this template

- Fork it or clone and change remote to your own repository


## Build PDF of the report:
```bash
./build.sh
```

