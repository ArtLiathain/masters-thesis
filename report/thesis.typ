#import "@preview/arkheion:0.1.1": arkheion, arkheion-appendices
#import "@preview/fletcher:0.5.8" as fletcher: diagram, node, edge
#import "@preview/fletcher:0.5.8": shapes
#import "@preview/dashy-todo:0.1.3": todo
#set page(
  paper: "us-letter", // IEEE/ACM standard
  margin: (x: 0.75in, y: 1in),
  columns: 1,
)

#set text(
  font: "Linux Libertine", // Closest to Times New Roman but looks better
  size: 12pt,
  weight: "regular",
)

#set par(
  justify: true,
  leading: 0.55em,     // Line spacing
  first-line-indent: 1.5em,
)
#show: arkheion.with(
  title: "Architectural Guardrails: An Opinionated Framework for Preventing Structural Decay in Greenfield Research Software",
  authors: (
    (name: "Art O'Liathain", email: "22363092@studentmail.ul.ie", affiliation: "University of Limerick"),
  ),
  keywords: ("Evolutionary Algorithms", "Genetic Programming", "Grammar Evolution", "Symbolic Regression", "Concurrent Computing"),
  date: "December 2025",
)

#show link: underline


#outline(
  title: "List of Figures",
  target: figure.where(kind: image)
)
#outline(
  title: "List of Tables", 
  target: figure.where(kind: table)
)
#pagebreak()

= Abstract


= Introduction
Core argument
- Research software is bad quality on average
- Static analysers is a step to improving quality but the current implementation is usually ignored due to fixes being suggestions
- Machine learning in this space is limited due to the volume of labelled data needed to create a model but there is precedent that it can improve performance over expert opinion
- An automatic approach to dataset creation using a volume based approach quickly identifying high risk files is the solution
- This solution is also tailored to pertain to research software particualrly to allow for the ml to learn the specific patterns there


=== Research question
Can opinionated objective rules be derived from historical and static analysis on code repositories

Do these rules notably increase code quality

#pagebreak()
= Background Research
== The current state of research software

Research software is interesting for a few different reasons, its software made to investigate new ideas and concepts. This comes with one very prominent issue, researchers who are leading the way to new concepts are generally not programmers by trade, 90% of scientists are self taught in programming @wilson_best_2014. 
This creates many issues, software is made without a care and it follows the simple check of #emph[It works on my computer]. This is best observed by @trisovic_large-scale_2022 where over 9000 R files were analysed from the Dataverse Project (http://dataverse.org/). In this study following good practice only 25% of projects were runnable and adding in code cleaning where dependencies were retroactively found and hardcoded paths were altered only 56% were able to be run.
This is an abysmal result, this result doesn't even compare if the results are consistent with the results this code produced. @trisovic_large-scale_2022 did note that journals are beginning to mandate better quality standards for code through research software standards to particularly allow for reproducibility to ensure the results published by the papers are correct. 

It is not enough, the crux of the issue is that software is made to fit a funding cycle without a care for the longevity of the codebase. Funding is fundementally not suited for research software @howison_sustainability_nodate notes that unlike commercial products where revenue grows linearly with users research grants are fixes and do not require or reward developers for user acquisition. Funding pushes research software to focus on itself with no incentives outside of a research paper output. A flawed approach which @howison_sustainability_nodate does address with two alternative solutions; Commercial models in which the research software itself transitions to a self sufficient project which keeps itself alive and peer-production models which is a similar approach as open-source. I agree that both of these models have merit, they allow research software to live past the funding cycle and continue to improve over time, there is one key issue with this approach. 


As 90% of researchers @wilson_software_2006 don't have prior programming experience, they need good clear guidance on how to create and maintain a large software project. Unfortunately research software guidance are laughable as broadly they only focus on software reproducibility @wilson_best_2014 @jimenez_four_2017 @marwick_computational_2017 @eglen_towards_2017 @wilson_software_2006 (Need to reword this later). Reproducibility is considered such a simple part of industry software that it isn't considered a metric to hit, it is a given yet in research software is it laid out as being the metric for best practice. Interestingly with all the focus on reproducibility  is no guidance on maintainability, coding standards or even basic structuring, the papers only if ever mention a vague sense of what should be done without and clear guidelines or criteria on how it should be done. A deficient that directly goes against the goals of @howison_sustainability_nodate in creating long living research software.


This is the gap my paper is seeking to fill, to create an opinionated tool that will ensure code is written well. This is a vital tool for research software as maintainable software is extendable software which will allow research to build upon each other more quickly and easily contributing to more focus put on solving new problems rather than remaking an old one to then use.

= Static Analysis
Static analysis in code is a storied field in which a primary focus has always been improving code quality. The tooling is imperative in modern engineering to allow for developers to see and fix maintainability and security issues in code. Unfortunately these tools are not prevalent in the research software space when it is most important as self taught developers do not have the mental model to identify the maintainability issue that plague codebases. 
One of the key ideas behind static analysis is abstraction. Abstraction refers to transformation of a program, called concrete program, into another program that still has some key properties of the concrete program, but is much simpler, and therefore easier to analyze.

== Lint

Lint was the first ever static analysis tool created for C the programmin language @johnson_lint_1978. 
Created for the C programming language the first linter was primary focused on an in the weeds analysis of code to identify errors cropping up syntactically and semantically in the codebase. Being the first of its kind it had limitations, it relied on the compilers front end infrastructure to run lexical and syntactical analysis on the input text, then creating an ascii file. Reading this ascii file the program was then able to identify issues in the codebase but it interpreted every token by token rather than a structured system. 
Lint was a pivotal moment for static code analysis as it paved the way for subsequent static analysis tooling, it garnered wide use and even named a subsection of tooling "linters". While lint is important, it is clear the limitations of static analysis only considering tokens is too much to justify using it to conduct project level analysis.

== Modern Approaches
Building from the groundwork Lint laid out, modern methods have changed significantly in how static analysis is done. These methods include data flow analysis, syntactic pattern matching, abstract interpretation, constraint based analysis etc... @gosain_static_2015. 
Using these methods a significantly more detailed analysis of programs can be conducted, where a move has been made from a token level analysis to a project level analysis, allowing approaches such as path analysis and reasoning about runtime behaviour. Yet even with this increase in analytic capabilities noise pollution in which there are too many error messages discouraging any actions being taking to remedy them, the same issue which plagued Lint @johnson_lint_1978 is still an issue to this day @dietrich_how_2017. Polluting the developer experience it creates friction and actively discourages developers form using the tools as it becomes more effort to sift through and find the real bugs than fixing them normally @johnson_why_2013. 
This leads to the conclusion that the research community due to the lack of experience that research software engineers have on average @wilson_best_2014 would have more difficulties differentiating fact from fiction and discouraging them from using static analysis at all.

This concretely highlights the difficulties of modern static analysis tooling in a research context. The focus on a general one size fits all tool means that it lacks the ability to be opinionated be default. The gap this presents is one where research software would have a bespoke set of curated rules reducing noise and guaranteeing improvements in code quality once executed.


== Maintainability
Maintainability is term used frequently in software engineering, there is no definite definition on what maintainability is but ISO25010 defines it as "The degree of effectiveness and efficiency with which a product or system can be modified to improve it, correct it or adapt it to changes in environment, and in requirements." It defines that the sub sections of maintainability are #emph[Modularity, Re-usability, Analysability, Modifiability, Testability]. @noauthor_iso_nodate
This is a very broad definition which simply means how easy can the system be modified for change. Due to the broadness of the definition it can easily be used to bring other non functional requirements as sub requirements ie. Flexibility, Adaptability, Scalability, Fault tolerance, Learn ability. The lack of clear testable outcomes for each quality in the ISO standard leads to a conceptual overlap where maintainability becomes a 'catch-all' category for non-functional requirements.

=== What does it mean to be maintainable?
To label code as maintainable, modularity is a key factor in that assessment. Modularity done correctly facilitates changeability through logical decomposition of functionality. This may seem simple but this area if done incorrectly can have long lasting consequences on the maintainability of a codebase. 
D.L Parnas presented a study revealing how easily developers could fall into the pitfall of designing the system in a human like manner @parnas_criteria_1972. Parnas argued that designing systems following a flowchart of logical operations is a method that creates code that is not resilient and while modular is highly coupled which hurts the maintainability of the codebase. The core purpose of modularisation shouldn't be encapsulating subroutines into modules. Design decisions that are likely to change should serve as the foundational points to creating modular code, allowing subroutines to compositions of modules. This will serve as a foundational idea on how maintainability will be measured in this paper.


=== Measuring Maintainability
Because maintainability is an abstract concept, various frameworks have attempted to reduce it to a concrete numerical metric. An early approach in this area is the Maintainability Index (MI), which calculates a score based on a weighted combination of Halstead Volume, cyclomatic complexity, lines of code (LOC), and comment density @oman_metrics_1992.
The formula typically utilized for this assessment is:$$MI = 171 - 5.2 \ln(V) - 0.23(G) - 16.2 \ln(LOC) + 50 \sin(\sqrt{2.46 \times C})$$Where $V$ represents Halstead Volume, $G$ is Cyclomatic Complexity, and $C$ is the percentage of comment lines. 


@oman_metrics_1992 validated this metric through feedback from 16 developers and empirical testing on industrial systems, including those at Hewlett-Packard and various defense contractors. This methodology provided a pragmatic tool for engineering managers to prioritize maintenance efforts by assigning a tangible value to code quality. 
While the MI provides a high-level overview of code density, it suffers from what can be described as "semantic blindness."
Metrics such as Cyclomatic Complexity and Halstead Volume analyze the control flow and token count of a file but fail to interpret the structural intent or the relationships between components. This is referred to as blind metrics and is a deficiency in this method. Utilising blind metrics allow for efficient and quick but it opens the door to gaming the system in a sense, where developers could reach extremely maintainable scores syntactically while semantically being unmaintainable, thereby abusing the formula.
An example of this is a script that maintains a high MI score due to low complexity and short length, yet contain architectural flaws such as "God Objects" or tight coupling to external datasets that inhibit reuse. The formula was a product of its time finely tuned to the teams and projects it was applied to, in the modern landscape the current formula would definitely not be accurate with how modern languages have evolved. Even though the formula could be adapted to a modern context it is clear that even a modernised version would suffer from blind metrics and is not something that should be used as only counting lines cannot yield results on the true quality of code. 



An approach that would apply in a more modern context was proposed by Xiao et al, of identifying architectural debt through evolutionary analysis of a codebase @xiao_identifying_2016. Where architectural debt can be used as a measure of maintainability albeit without a clear score. The paper proposed that there are four key architectural patterns that are the main proponents of ATD, Hub, Anchor Submissive, Anchor Dominant, and Modularity Violation. These patterns are all based on evolutionary dependencies between files, particularly those that are correlated only through commits and lack structural commonality.

Explain briefly the four types with images if possible

This was measured by preforming a pseudo longitudinal study on large open source codebases, using tools such as Understand and Titan to calculate commit coefficient between every file. A metric which relates to the chance of a commit on one file will require a commit on another. An example would be given files A, B and C, with a commit history of {A,B} and {A,C} the commit coefficients in relation to A are there is a 100% chance if B or C is modified A will be also, but if A is modified there is only a 50% chance that B or C will be modified.
This would then be called a fileset, this would be extrapolated over the entire commit history and codebase creating many filesets. This would allow the tool to extract semantic relations between files that are not apparent structurally. The study was able to compound this effect by measuring the number of commits labeled as bug fixes against the number of feature commits, this allows simple data analytics to measure the amount of maintenance debt that each fileset would have. 
Using this method, high maintenance filsets can be labelled and evaluated as a quantitative metric. This is a strong contender for a evaluation metric but there is a caveat, that is it is reliant on good commit messages and issue tracking which is not commonly seen in research code. Meaning that a new approach that is agnostic to the quality of the codebase is required to.



= Identifying maintainability hotspots
== Code churn

Code churn is the rate of change of lines of code in a file. A file with high churn might have an incredibly high number of additions and deletions while remaining small. This is an ATD hotspot when assessing points of interest in terms of maintainability in codebases. @farago_cumulative_2015 is paper which assess how code code churn affects the maintainability of codebases, by assessing the code churn of files in large codebases, achieving this by calculating the sum total of lines added and deleted on every file. 
Supplementing this the maintainability of the code was measured using ColumbusQM probabilistic software quality model @noauthor_pdf_nodate. The paper was able to assess the levels of maintainability per file in proportion to the amount of code changes, drawing the conclusion that code that has high churn is code in which it is harder to maintain. This conclusion makes sense, it reaffirms that poorly architect ed code will need more changes which increases the maintainability burden of the code. The method in which the code churn was measured is quite useful and will be the approach taken for this study.

Unfortunately there is one key issue with this paper, the use of ColumbusQM for assessing the maintainability of the codebase. 
ColumbusQM is a statistical machine which takes metrics such as lines of code, styling, coupling, number of incoming invocations etc.. Aggregates them which then computes a numerical output. The metrics such as styling are something so subjective and minor that the inclusion as one of the key metrics in the evaluation underscores the whole statistical model. Taking into account all of the evaluation metrics this is a case of blind assessment where structural relations or evolutionary dependencies are not measured and only a snapshot of the codebase is measured. Compounding on this the evaluation metric for the output of the statistical model was based on developer feedback and ideas, which is unreliable and subjective. 

Taking these points into account, the key takeaway from @farago_cumulative_2015 is the work on code churn particularly the methodology but the conclusions in regards to maintainability will only be used as an idea within this study.



= The current problem with static analysers(maybe ghost source? Its quite a good one)
When discussing the issues with static analysers before, the warning pollution was a key issue detracting from the user experience of static analysers @johnson_why_2013. It pushes developers from tackling problems as false positives are a painful process to sift through. 
This paper poses that the key issue with the analysers in the case of false positives is not the rules but how they were derived. Historically most academic research in software engineering has required significant empirical evidence to be considered by industry @weyuker_empirical_2011, yet the current landscape is dominated by expert analysed rules. 
Rules from experts can only be taken as subjective opinion, contributing to warnings over a hard line on what should be allowed in codebases.
Previous research @bielik_learning_2017 has shown that machine learning in the context of statics analysis can lead to improvements on inference rules in javascript. Taking a sample set of 20,000 javascript test cases the study used a decision tree based algorithm to create a set of rules analytically that had an increased efficiency in predicting the outcomes of corner cases over the previously established expert rules. Now this did rely on a deterministic objective problem of runtime inference, which is not applicable to this problem but it set a precedent that the expert derived rules are not a perfect set of rules and that empirically derived rules have room to improve the quality of static analysis tooling.


= Machine learning
Machine learning is a pivotal technology within the context of this paper. It allows rigorous statistical analysis to be carried out to create objective metrics for the maintainability rules. Machine learning is based on the principle of minimising loss, creating a method of training an algorithm to optimise for the lowest loss through gradient descent allows for the code to be self improving within the problem domain. There are many approaches and models available but due to an emphasis put on human readability decision trees and random forest best fits this use case. 


== Decision Trees
Decision trees are a standout choice when it comes to an interpretable machine learning models. They consistently rank highly within classifier models while still supporting rule extraction which aids readability. Decision trees represent classifiers as hierarchical IF-THEN rules, where each root-to-leaf path corresponds to a conjunction of feature thresholds that directly maps to executable static analysis checks.
A concrete example of this would be IF nesting > 3 THEN unmaintainable, in the case of nesting being over 3 then the code would be labeled unmaintainable. This simplistic example would be scaled up much larger with many more decision and leaf nodes creating a classifier based off simple decisions at each step. 
Naturally there are some drawbacks to this approach, most prominently is the overfitting problem. An overfitted model reflects the structure of the training data set too closely. Even though a model appears to be accurate on training data,, it may be much less accurate when applied to a current data set @khoshgoftaar_controlling_2001.
This issue restricts their usage in current maintainability prediction methods, @bluemke_experiments_2023 shows that decisions trees can prove useful in a baseline analysis of maintainability predictors.
Despite limitations, DT rules provide the simple interpretability missing from random forests, enabling manual validation of learned static checks against domain knowledge. Pruning or ensemble averaging in random forests addresses overfitting to a degree while retaining path-derived rules for analysis. @quinlan_c45_1993
== Random Forest
Random forest is an approach derived from decision trees and arguably one of the most popular classifier machine learning approaches. Relying on creating a set of decision trees (called a forest) each of which vote on the outcome to choose a class for the classifier, it creates a method that leverages the law of large numbers to deal with the overfitting problem in decision trees @breiman_random_2001 in addition to increasing accuracy. The method in which overfitting can be largely ignored is that there are many trees voting, every one could potentially be overfitted yet the whole forest remains generalised to the problem as every tree would be overfitted to a different feature.
Random forests average better results theoretically @breiman_random_2001 and practically in the field of maintainability research @bluemke_experiments_2023 over decision trees. The key drawbacks to random forests are both the increased computational costs and the black box view when it comes to readability. 
Modern research @haddouchi_forest-ore_2025 has made progress in creating methods of rule extraction for random forests as before the size of the forests rendered them as a black box approach. @haddouchi_forest-ore_2025 focused on deriving a rule ensemble based on the calculated weighted importance of the trees. Allowing for a dimension reductionality method to be applied to the forest. Within the study a factor of 300x reduction to the trees per class was observed while retaining a 93% accuracy compared to the full forests 95% accuracy. Leveraging this rule extraction method for random forests is pivotal to create a rule set that properly generalises across projects while retaining the core idea and performance of the model and keeping with the core ideal of readability.


== Applications to maintainability detection
The application of machine learning to maintainability detection enables a data-driven approach to identifying code structures that hinder long-term quality and evolution. Traditional static analysis methods rely on human-curated heuristics, which often fail to generalise across diverse projects or languages. In contrast, machine learning can infer maintainability rules directly from empirical evidence, allowing patterns of poor or high maintainability to emerge statistically rather than being predefined. Which has been shown to outperform human experts @borg_ghost_2024.
SotA models achieve impressive results in classification of low maintainability files: @bertrand_replication_2023 achieved a 82% F1 score with a AdaBoost classifier and @bluemke_experiments_2023 achieved an F1 score of 93% using a potentially more readable random forest classifier. Both of these results are very impressive although a key fault with both are the reliance on human experts to create a ground truth of annotated data. PROMISE @noauthor_pdf_nodate-1 and MainData @schnappinger_defining_2020 were the datasets used in both these studies, relying on historical evidence along with expert opinion both fall victim to two issues, scale and objectivity. Datasets such as these are an arduous process and in the end result in a dataset that is by design older and not able to reflect the current coding landscape. Technology changes too quickly and datasets such as these rely on immutability to allow models to learn whereas the landscape is anything but, which was the key motivation for an algorithmic approach to labelling and analysis to create a dataset based on current codebases at scale.

A crucial step in applying these machine learning to maintainability detection is the transformation of source code into a machine-readable representation. Abstract Syntax Trees (ASTs) provide this structure, encoding the syntactic and hierarchical relationships between program elements. By traversing or analysing the AST, a wide range of numerical and categorical features can be extracted: such as nesting depth, branching density, or average method size, which serve as the input features for machine learning models @bertrand_building_2022.
Various learning models can then be trained on these AST-derived metrics to classify code fragments according to maintainability characteristics. 

= Methodology

This study follows an empirical, data-driven pipeline to derive static analysis rules from real-world research software. The process is divided into four primary phases. First, we aggregate a dataset of C++ repositories from the Journal of Open Source Software (JOSS). Second, we employ a custom-built tool to extract evolutionary coupling metrics and calculate a Hub Score for each file, providing an objective "High Risk" or "Low Risk" label. Third, these labeled files are used to train a Decision Tree model to identify the structural characteristics of high-risk code. Finally, the logical paths within the trained model are translated into human-readable static analysis rules, bridging the gap between historical developer behavior and proactive code quality standards.

== Dataset Selection and Acquisition
The primary goal of this paper is to derive objective rules to raise the level of coding standards in the research software scene. This lends itself to using research software as the dataset. JOSS @noauthor_build_nodate is an open source website which allows for researchers to submit open source research project, this is a dataset which provides a curated set of peer-reviewed research software, ensuring domain-specific relevance to use as each repository is guaranteed to be a research repo. To ensure sufficient architectural complexity, projects were filtered using a purposive sampling strategy: a minimum of 4 collaborators and a commit history between 100 and 5,000 commits. C++ was selected as the target language due to its prevalence in high-performance research computing and the inherent risk associated with its low-level memory operations. Thus creating a dataset which comprises sufficiently complex projects all of which have academic grounding for evaluation.

== Data Labeling 
To categorize the dataset,automated labeling heuristics were utilised rather than traditional expert manual review. While expert opinion is often the standard for establishing "ground truth," it is difficult to scale across thousands of research repositories. Instead, we analyzed the historical behavior of each file to assign a Hub Score: a composite metric that measures a file’s "entanglement" within the project. This score is calculated based on three key factors: the number of coupled files, the average coupling ratio, and the file's overall code churn.We gathered these metrics by running a custom analysis tool over the entire commit history of each repository. The tool constructs a Co-change Graph where, Nodes represent individual files and Edges represent shared commits between files.
The graph allows us to evaluate the coupling of each file by comparing the number of times it was modified alongside other files versus the number of times it was modified in isolation.
(INSERT FORMULA)
As noted by @xiao_identifying_2016, focusing primarily on modularity violations ignores other potential risk factors. However, this approach is justified by the unique scale of our study. Unlike traditional, manually-curated datasets such as PROMISE @noauthor_pdf_nodate-1 or MainData @schnappinger_defining_2020, our automated "broad-spectrum" filtering allows us to identify the most critical offenders across a significantly larger volume of data. By combining these repositories, we created a dataset that is both objective and massive in scope, providing a high-quality surface area for supervised learning using "High Risk" and "Low Risk" labels.== Feature Selection

== Model Implementation and Training

== Rule Derivation Process

== Evaluation Framework


#bibliography("references.bib")

