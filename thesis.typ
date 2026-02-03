#import "@preview/arkheion:0.1.1": arkheion, arkheion-appendices
#import "@preview/fletcher:0.5.8" as fletcher: diagram, node, edge
#import "@preview/fletcher:0.5.8": shapes
#import "@preview/dashy-todo:0.1.3": todo

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

= Introduction
This is just notes for into and paper structure

- Talk about code quality as a whole
- Talk about issues in research funding cycle
- Talk about how pervasive technical debt is
- Talk about reproducibility paper -> Mention the affect this has on the validity of papers

The core idea of the debt analysis is a 3 step process Identify high risk/churn code areas, use a tool maybe machine learning to analyse and understand the code, derive rules from both texts such as architectural smells that are correlated to high #emph[debt interest] @xiao_identifying_2016 

=== Research question
Can opinionated objective rules be derived from historical and static analysis on code repositories

Do these rules notably increase code quality

=== Thoughts on structure
Instead of one literature review, I'm almost thinking of stages so I do one review now for identification methods and then another for creating an AST analyser as well as the methodology since both feel they could've been a mini thesis in their own right and if I don't do it this way i feel it'll be very very front loaded


= Background on Identifying technical debt
- Maintainability
- Technical Debt
- State of Research software
- Importance of architectural debt
- Current methods to identify debt (Smells and @xiao_identifying_2016)
- 
== Maintainability
Maintainability is term used frequently in software engineering, there is no definite definition on what maintainability is but ISO25010 defines it as "The degree of effectiveness and efficiency with which a product or system can be modified to improve it, correct it or adapt it to changes in environment, and in requirements." It defines that the sub sections of maintainability are #emph[Modularity, Re-usability, Analysability, Modifiability, Testability]. 
This is a very broad definition which simply means how easy can the system be modified for change. Due to the broadness of the definition it can easily be used to bring other non functional requirements as sub requirements ie. Flexibility, Adaptability, Scalability, Fault tolerance, Learn ability. One could easily make an argument for most sub categories and even categories defined within the ISO 25010 standard @noauthor_iso_nodate to be sub sections within maintainability with maintainability being the true goal of non functional requirements.

This naturally comes with the problem of how does one measure maintainability? There have been many different approaches taken to measuring maintainability. 


As this is a very wishy washy definition it makes more sense for this paper to define maintainability as "The inverse amount of technical debt to lines of code". This is a much simpler defintion that is quantifiable, although it requires that technical debt be explained in detail for it to be truly understood.

Technical debt (TD) was first defined as "Shipping first time code is like going into debt. A little debt speeds development so long as it is paid back promptly with a rewrite... The danger occurs when the debt is not repaid. Every minute spent on not-quite-right code counts as interest on that debt. Entire engineering organizations can be brought to a stand-still under the debt load of an unconsolidated implementation, object-oriented or otherwise." By Ward Cunningham @noauthor_c2comdocoopsla92html_nodate. 
In the following years research has shown TD is not a singular type of problem and there are many forms to it, the five most prevalent types of TD are #text(weight:"bold")[Design debt, Test debt, Code debt, Architecture debt and Documentation debt] this was extracted from 700+ surveys across 6 countries @ramac_prevalence_2022. The artifacts used to identify design debt, code debt and architectural debt have significant overlap and these artifacts exhibit behaviours similar to Architectural Technical Debt (ATD)@xiao_identifying_2016. 
Similarly to Maintainability, this paper defines Technical debt in regards to design, architecture and code as ATD and will be the primary focus of this paper.



Because maintainability is an abstract concept, various frameworks have attempted to reduce it to a concrete numerical metric. A foundational approach in this area is the Maintainability Index (MI), which calculates a score based on a weighted combination of Halstead Volume, cyclomatic complexity, lines of code (LOC), and comment density @oman_metrics_1992.
The formula typically utilized for this assessment is:$$MI = 171 - 5.2 \ln(V) - 0.23(G) - 16.2 \ln(LOC) + 50 \sin(\sqrt{2.46 \times C})$$Where $V$ represents Halstead Volume, $G$ is Cyclomatic Complexity, and $C$ is the percentage of comment lines.
The Oman study validated this metric through feedback from 16 developers and empirical testing on industrial systems, including those at Hewlett-Packard and various defense contractors. This methodology provided a pragmatic tool for engineering managers to prioritize maintenance efforts by assigning a tangible value to code quality.2.3 The Limitations of "Blind" MetricsWhile the MI provides a high-level overview of code density, it suffers from what can be described as "semantic blindness."
Metrics such as Cyclomatic Complexity and Halstead Volume analyze the control flow and token count of a file but fail to interpret the structural intent or the relationships between components. This represents a significant gap in the literature regarding Architectural Technical Debt (ATD).For example, a script may maintain a high MI score due to low complexity and short length, yet contain architectural flaws such as "God Objects" or tight coupling to external datasets that inhibit reuse. Because the MI only evaluates the "bulk" of the code rather than its structure, it cannot identify these high-interest debt artifacts. This limitation justifies the shift toward Abstract Syntax Tree (AST) parsing and semantic pattern recognition. Instead of aggregating raw counts, a semantic approach allows for the identification of specific code patterns that fundamentally impede long-term maintainability.

A foundational study on calculating maintainability took to an approach of combining Halstead Volume, comment count, cyclomatic complexity and lines of code (LOC) @oman_metrics_1992. This study posed that a maintainability score can be calculated from proportional additions of these metrics. This was backed up by both feedback from 16 software developers as well as empirical analysis of the metric being used on industrial codebases such as Hewlett-Packard and defense contractors. 
This was a reasonable method of assessment of code as it both gave a concrete numerical metric to code quality allowing code to be prioritised quickly. 






Technical debt is a pervasive problem in research software development @hassan_characterising_2025 where does it compiles is the only quality tracked in the codebase. This means that technical debt accrues significant interest as time increases 


#bibliography("references.bib")

