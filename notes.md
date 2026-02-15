# Cumilative Code churn.
Cumilative Code churn -> Number of lines deleted addd or modified from the source


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

