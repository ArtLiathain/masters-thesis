# Cumilative Code churn Impact on maintainability
Cumilative Code churn -> Number of lines deleted added or modified from the source

Key points, they conclude that modifying files already modified a lot increases the mainatinability burden -> This makes sense, a file that changes often violates SOLID and other concepts
This is predicated on the probabilistic software quality model so hard to substantaite

Code churn measured is a good idea to show what files are undergoing rapid change

### Good
-

### Bad
- Referencing ISO as a standard

# A Probabilistic Software Quality Model
I dont like it, relevant as cumilative code churn uses this as its basis for maintainability
It primarily focuses on creating a mathematical representation on what maintainability looks like in a codebase.
It derives a "goodness function" to determine how each metric is doing which can be compared across versions
It acknoledges that the graph shouldn't be used generally as it needs to be context dependent

## Good
- ADG has logical sense with the aggregate metrics
- Evaluated on large sample set
- 

## Bad
- Used surveys from managers testers sw.engs, consultants and testers for weighting on ADG for maintainability
- Calculating maintianability as a snapshot without historical context
- Style is used as a maintainability metric nuff said
- LOC used as a hard metric 
- For validation developers were asked opinions on the codebase, which did differ from the model showing that both were unreliable
# A Cost Model Based on Software Maintainability
A paper predicated on the one above, i dont want to double dip but due to issues in the above this one has the same


# Finishing the identifying high churn files section
- How has code churn been identified before
- The issue with totally blind metrics
- How churn should be used as a flag for investigation over the final answer
- Maintainability is too abstract to be broken down into a formula


# Static Analysis
Talk about how modern static analysis works through AST, how they snapshot the code to give 
If intuition is correct the approach si that the groundwork for these tools are old, but the modernisation of them is generally higer level concept particularly about reducing false positives
This is where both a modernisation of approach using temporal analysis and specific targeted rules can apply to this context.

## Abstract Interpretation
- A paper focused on how a lattice representation for AST can be used for static anlysis through formal proofs of code -> Very old refernce to modern day static analysis

## Datadog report 
- Advantage of shift left

## Why don’t software developers use static analysis tools to find bugs?
- False positives and methods of display are barries to use 
- 20 developers surveyed
- Developers like the idea more than the use
- Fits small codebases better larger ones have too much noise
- Customisation is important



# Wasted time
- Stripe 2018 study 17hrs a week devs spend fixing bad code

## For later
- Martin fowler about small code changes -> not the best solution but what researchers need

# Learning a static analyzer from data
https://files.sri.inf.ethz.ch/website/papers/cav17-pa.pdf

### Focus 
To create rules for a static analyser to better help inference in cases of purely static analysis inference

### Method
Take a 20k sample set of js test cases, using the result as ground truth train a decision tree model to identify the result without running the code
Combining this with an oracle with a similar principle to GANs where the oracle generates counterexamples based off incorrect results from the discriminator


### Output and things of note
This does set a good precedent for rules generated empirically that can trump expertly derived rule
It outperformed metas Flow model at the time

### Issues
None so far the paper seems solid enough, but the only purpose in mentioning it is that it stands to reason that other areas could benefit from empirically derived rules over expert opinions


# 


# When reading papers 
What is the aim of the paper

How did they do what they've done

What were the results

Conclusions

What do you think the results mean



Outline methodology

# Dimensionality Reduction
PCA


# Proofreading
- Reading aloud
- Change font size
- Think of debating a 4 year old why why why

# Lit review
- After methodology add purposive paragraphs to direct towards the idea i have


Fixes
- Check wilson 2006 and 2016 sources
- Static analysis -> Why are we going for static analysis justify why we want to do this. Talk about training and why we're not doing other approaches



# Feedback 2
- 8.1 -> JOSS cite is the github homepage (Cite their webpage)
- TODO NOTES -> Convert into a formatted note ( Typst approach flag for TODO )
- Do the introduction give some context motivation and framing develop it out -> Bulletpoint skeleton for today
- End of 6 -> Need a cleaner bridge -> Need to highlight the fact that expert rules -> Dont dimiss their approach, given a solid dataset their is precedent for advantages over expert rules 
- Machine learning - Good 
- Develop a coherent argument throughout the paper -> Need a bit of a flow now.
- Hub score needs a justification from both literature and comparison to real world analysis


# Feedback 3
- https://piechowski.io/post/git-commands-before-reading-code/
The Git Commands I Run Before Reading Any Code
Five git commands that tell you where a codebase hurts before you open a single file. Churn hotspots, bus factor, bug clusters, and crisis patterns. Repository analysis over dataset creation
- Chidamber and Kemerer (CK) metrics
- https://arxiv.org/abs/2604.04809 software energy smells
- https://www.computer.org/product/education/professional-software-engineering-master-certification
- https://www.computer.org/product/education/professional-software-developer-certification


# Feedback 4
- Introduction should be built out more (2-3)
- FAIR Principles
