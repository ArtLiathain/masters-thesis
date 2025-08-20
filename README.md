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
- Some PDF generation flags for xelatex
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
- Fork repo or clone and change remote to your own repository
- Set up a fine-grained personal access token with R/W permissions on Contents and Actions on new repo
- Add it to new repo > Settings > Secrets > Actions > Repository secrets as `CI_PAT`
- Add your personal details to env vars in `.github/workflows/build.yaml`
- `cz commit` instead of `git commit` (!IMPORTANT!)
- Push to branch and pr, or push straight to `main`. Build triggers on changes to `main`
- Ensure build is successful, creates a release and tags correctly.

## Local development
- Make changes
- Run `build.sh` to build the PDF locally
- Open `thesis.pdf` in your PDF viewer to see the changes
- `cz commit` NOT `git commit`

## Pro tips
- Use `clean.sh` to remove intermediate files. 
This can sometimes fix build issues, particularly with citations.
- If you push straight to `main`, commit to main again without pulling, 
and try to push again, you'll get an error. 
Fix with `git pull --rebase` and then push again.
