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


=== Research question
Can opinionated objective rules be derived from historical and static analysis on code repositories

Do these rules notably increase code quality

=== Thoughts on structure

The core idea of the debt analysis is a 3 step process Identify high risk/churn code areas, use a tool maybe machine learning to analyse and understand the code, derive rules from both texts such as architectural smells that are correlated to high #emph[debt interest] @xiao_identifying_2016 

= Background on Identifying technical debt
- Current methods to identify debt 
  - smells
  - machine learning
- Impact debt has on code in the long run
- How do the suggestions work
- AST and how they work
  - Ruff
  - Sonarqube etc
- (MAYBE) Talk about machine learning in pattern analysis for detecting high bug spots for the code and repeating issues.

== What needs to be completed now
- Static Analysis methods
- Code churn detection




#pagebreak()
== The current state of research software

Research software is interesting for a few different reasons, its software made to investigate new ideas and concepts. This comes with one very prominent issue, researchers who are leading the way to new concepts are generally not programmers by trade, 90% of scientists are self taught in programming @wilson_best_2014. 
This creates many issues, software is made without a care and it follows the simple check of #emph[It works on my computer]. This is best observed by @trisovic_large-scale_2022 where over 9000 R files were analysed from the Dataverse Project (http://dataverse.org/). In this study following good practice only 25% of projects were runnable and adding in code cleaning where dependencies were retroactively found and hardcoded paths were altered only 56% were able to be run.
This is an abysmal result, this result doesn't even compare if the results are consistent with the results this code produced. @trisovic_large-scale_2022 did note that journals are beginning to mandate better quality standards for code through research software standards to particularly allow for reproducibility to ensure the results published by the papers are correct. 

It is not enough, the crux of the issue is that software is made to fit a funding cycle without a care for the longevity of the codebase. Funding is fundementally not suited for research software @howison_sustainability_nodate notes that unlike commercial products where revenue grows linearly with users research grants are fixes and do not require or reward developers for user acquisition. Funding pushes research software to focus on itself with no incentives outside of a research paper output. A flawed approach which @howison_sustainability_nodate does address with two alternative solutions; Commercial models in which the research software itself transitions to a self sufficient project which keeps itself alive and peer-production models which is a similar approach as open-source. I agree that both of these models have merit, they allow research software to live past the funding cycle and continue to improve over time, there is one key issue with this approach. 


As 90% of researchers @wilson_software_2006 don't have prior programming experience, they need good clear guidance on how to create and maintain a large software project. Unfortunately research software guidance are laughable as broadly they only focus on software reproducibility @wilson_best_2014 @jimenez_four_2017 @marwick_computational_2017 @eglen_towards_2017 @wilson_software_2006 (Need to reword this later). Reproducibility is considered such a simple part of industry software that it isn't considered a metric to hit, it is a given yet in research software is it laid out as being the metric for best practice. Interestingly with all the focus on reproducibility  is no guidance on maintainability, coding standards or even basic structuring, the papers only if ever mention a vague sense of what should be done without and clear guidelines or criteria on how it should be done. A deficient that directly goes against the goals of @howison_sustainability_nodate in creating long living research software.


This is the gap my paper is seeking to fill, to create an opinionated tool that will ensure code is written well. This is a vital tool for research software as maintainable software is extendable software which will allow research to build upon each other more quickly and easily contributing to more focus put on solving new problems rather than remaking an old one to then use.


--- Point of the section
- Highligh current analysis methods
- Talk about how this predominantly has a storied path
- Issues with noisy error and false positives
- How that when applied to research means that generally errors will slip through the cracks

= Static Analysis
Static analysis in code is a storied field in which a primary focus has always been improving code quality. The tooling is imperative in modern engineering to allow for developers to see and fix maintainability and security issues in code. Unfortunately these tools are not prevalent in the research software space when it is most important as self taught developers do not have the mental model to identify the maintainability issue that plague codebases. 

One of the key ideas behind static analysis is abstraction. Abstraction refers to
transformation of a program, called concrete program, into another program that
still has some key properties of the concrete program, but is much simpler, and
therefore easier to analyze [5]. Over-approximation results when there are more
behaviors in the abstraction than in the concrete system [5]. Under-approximation,
on the other hand, deals with fewer behaviors than in the concrete system.
Static analysis can be sound or unsound. Soundness guarantees that the information computed by the analysis holds for all program executions, whereas
unsound analysis does not [11]. Static analysis can be made path, flow, and context
sensitive by making it to distinguish between paths, order of execution of statements, and method call (call site), respectively [12]. Precision of an analysis
approach can be measured by classifying the analysis results into three categories
[13]: false positives, i.e., nonexistent bugs are reported as potential bugs; false
negatives, i.e., bugs that are undiscovered; and true positives, i.e., true and discovered bug. Efficiency is related to computational complexity or cost pertaining to
space and time requirements of the algorithms used in the analysis [5]. Precision
and efficiency are related to each other, i.e., a precise analysis is more costly and
vice versa.

== Lint the progenitor

Before modern day static analysis using Abstract syntax trees, linters were at the forefront of code quality analysis @johnson_lint_1978. 
Created for the C programming language the first linter was primary focused on an in the weeds analysis of code to identify errors cropping up syntactically and semantically in the codebase. Being the first of its kind it had limitations, it could only parse the code as a string and relied on regex to guess if the data flow allowed certain structures to be called. The tools available limited its uses but it was still able to identify key issues such as warnings regarding suspicious type conversions, non-portable constructs, and unused or uninitialized variables. Lint was a pivotal moment for static code analysis as it paved the way for subsequent static analysis tooling, it garnered wide use and even named a subsection of tooling "linters". While lint is important, it is clear the limitations of static analysis using regex is too much to justify using it to analyse semantic structure in codebases and more sophisticated tools are required.

== Modern Approaches
=== Data Flow analysis
The most popular static analysis technique. By constructing a graph-based representation of the program, called a control flow graph, and writing data flow equations for each node of the graph. These equations are then repeatedly solved to calculate output from input at each node locally until the system of equations stabilizes or reaches a fixed point. The major dataflow analyses used are reaching definitions (i.e., most recent assignment to a
variable), live variable analysis (i.e., elimination of unused assignments), available
expression analysis (i.e., elimination of redundant arithmetic expressions), and very
busy expression analysis (i.e., hoisting of arithmetic expressions computed on multiple paths) [2]. At each source code location, data flow analysis records a set of facts
about all variables currently in scope. In addition to the set of facts to be tracked, the
analysis defines a “kills” set and a “gens” set for each block. The “kills” set describes
the set of facts that are invalidated by execution of the statements in the block, and the
“gens” set describes the set of facts that are generated by the execution of the statements in the block. To analyze a program, the analysis tool begins with an initial set of
facts and updates it according to the “kills” set and “gens” set for each statement of the
program in sequence. Although mostly used in compiler optimization @kam_global_1976 data
flow analysis has been an integral part of most static analysis tools @gosain_static_2015.


This technique was formalized by Cousot and Cousot [22]. It is a theory of
semantics approximation of a program based on monotonic functions over ordered
sets, especially lattices [23]. The main idea behind this theory is as follows: A
concrete program, its concrete domain, and semantics operations are replaced by an
approximate program in some abstract domain and abstract semantic operations.
Let L be an ordered set, called a concrete set, and let L′ be another ordered set,
called an abstract set. A function α is called an abstraction function if it maps an
element x in the concrete set L to an element α(x) in the abstract set L′. That is,
element α(x) in L′ is the abstraction of x in L. A function γ is called a concretization
function if it maps an element x′ in the abstract set L′ to an element γ(x′) in the
concrete set L. That is, element γ(x′) in L is a concretization of x′ in L′. Let L1, L2, L′
1, and L′2 be ordered sets. The concrete semantics f is a monotonic function from
L1 to L2. A function f′ from L′1 to L′2 is said to be a valid abstraction of f if for all x′
in L′1, (f ∘ γ)(x′) ≤ (γ ∘ f′)(x′). The primary challenge to applying abstract interpretation is the design of the abstract domain of reasoning [24]. If the domain is too
abstract, then precision is lost, resulting in valid programs being rejected. If the
domain is too concrete, then analysis becomes computationally infeasible. Yet, it is
a powerful technique because it can be used to verify program correctness properties and prove absence of errors [25, 26].


=== Needs to change just basically what i do believe and need to see if the sources back it up
As the base approach and tooling have largely remained static the large innovations in modern day static analysis is rules creation and fine tuning. Approaches like adding temporal analysis or creating more accurate rules that bring up less false positives when flagging bad code through the use of machine learning are key innovations. Due to the nature of these tools they've been created assuming a base level of developer confidence and putting trust in developers to know when they can and should break rules. This flexibility is a key component as to why these tools are successful and exactly why in a research context where code quality cannot reach basic standards, it is allowing developers too much freedom and enabling bad code. 
Another key insight is that these tools have been developed with industry in mind, a gap is a direct focus on research software where different practices are normally employed and a more fine tuned approach is required. 


== Code churn

Code churn is the rate of change of lines of code in a file. A file with high churn might have an incredibly high number of additions and deletions while remaining small. This is an ATD hotspot when assessing points of interest in terms of maintainability in codebases. @farago_cumulative_2015 is paper which assess how code code churn affects the maintainability of codebases, by assessing the code churn of files in large codebases, achieving this by calculating the sum total of lines added and deleted on every file. 
Supplementing this the maintainability of the code was measured using ColumbusQM probabilistic software quality model @noauthor_pdf_nodate. The paper was able to assess the levels of maintainability per file in proportion to the amount of code changes, drawing the conclusion that code that has high churn is code in which it is harder to maintain. This conclusion makes sense, it reaffirms that poorly architect ed code will need more changes which increases the maintainability burden of the code. The method in which the code churn was measured is quite useful and will be the approach taken for this study.

Unfortunately there is one key issue with this paper, the use of ColumbusQM for assessing the maintainability of the codebase. 
ColumbusQM is a statistical machine which takes metrics such as lines of code, styling, coupling, number of incoming invocations etc.. Aggregates them which then computes a numerical output. The metrics such as styling are something so subjective and minor that the inclusion as one of the key metrics in the evaluation underscores the whole statistical model. Taking into account all of the evaluation metrics this is a case of blind assessment where structural relations or evolutionary dependencies are not measured and only a snapshot of the codebase is measured. Compounding on this the evaluation metric for the output of the statistical model was based on developer feedback and ideas, which is unreliable and subjective. 

Taking these points into account, the key takeaway from @farago_cumulative_2015 is the work on code churn particularly the methodology but the conclusions in regards to maintainability will only be used as an idea within this study.

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






== Technical Debt

Technical debt (TD) was first defined as "Shipping first time code is like going into debt. A little debt speeds development so long as it is paid back promptly with a rewrite... The danger occurs when the debt is not repaid. Every minute spent on not-quite-right code counts as interest on that debt. Entire engineering organizations can be brought to a stand-still under the debt load of an unconsolidated implementation, object-oriented or otherwise." By Ward Cunningham @noauthor_c2comdocoopsla92html_nodate. 
In the following years research has shown TD is not a singular type of problem and there are many forms to it, the five most prevalent types of TD are #text(weight:"bold")[Design debt, Test debt, Code debt, Architecture debt and Documentation debt] this was extracted from 700+ surveys across 6 countries @ramac_prevalence_2022. The artifacts used to identify design debt, code debt and architectural debt have significant overlap and these artifacts exhibit behaviours similar to Architectural Technical Debt (ATD)@xiao_identifying_2016. 
Similarly to Maintainability, this paper defines Technical debt in regards to design, architecture and code as ATD and will be the primary focus of this paper.

Technical debt is a pervasive problem in research software development @hassan_characterising_2025 where "does it compile" is the only quality metric tracked in codebases. This means that technical debt accrues significant interest as time increases. 

== Dealing with technical debt 
When dealing with technical debt there are two 
- How to identify with smells
- How do auto suggestions work look at tools



#bibliography("references.bib")

