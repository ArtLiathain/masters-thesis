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


# When reading papers 
What is the aim of the paper

How did they do what they've done

What were the results

Conclusions

What do you think the results mean



Outline methodology

# Dimensionality Reduction
PCA
