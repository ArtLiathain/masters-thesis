# UL Thesis Template

Based on [original UL thesis template](https://www.overleaf.com/latex/templates/university-of-limerick-thesis-template/dhhwcbhtjbcc) by Guiseppe Torre.


# Purpose

This repo lightly modifies the tooling and setup of the original template.

These modifications enable:
- Local development of a UL thesis in LaTeX
- Semantic versioning of the thesis document
- Automated building of the document and release with GitHub Actions

# Changes and Additions:
- Citation style changed to IEEE
- Some changed to PDF generation flags to use xelatex
- Commitizen for semantic versioning
- Github Action to build document, save version

# Tooling

## Required

Deviation from these will break the build process:
- Semantic versioning: Commitizen [commitizen](https://commitizen-tools.github.io/commitizen/) (I recommend using it through uv)
- Texlive: xelatex (to build pdf) and publishers (for IEEE style) `sudo apt install texlive-xetex texlive-publishers`

## Recommended

I found these quite good
- LSP: texlab (or whatever plays well with your editor)
- Viewer: sumatra PDF (hot reloading is great, stays in the same place of the document)

# How to use this template

## Initial setup
- Fork it or clone and change remote to your own repository
- Set up a fine-grained personal access token with R/W permissions on Contents and Actions on this repo
- Add it to Settings > Secrets > Actions > Repository secrets as `CI_PAT`
- Add details to `.github/workflows/build.yaml`
- `cz commit` instead of `git commit` (!IMPORTANT!)
- Push to branch and pr, or push straight to `main`. Build triggers on changes to `main`
- Ensure build is successful, creates a release and tags correctly

## Local development
- Make changes
- Run `./build.sh` to build the PDF locally
- Open `thesis.pdf` in your PDF viewer to see the changes
- `cz commit` NOT `git commit`
