
SLOPCHOP v1.2
THE LAW OF LOCALITY
A Topological Integrity Framework for High-Integrity Systems
Design Specification & Implementation Brief
Version 1.2.0 — December 2024

Preamble: Purpose and Audience
This document serves two audiences: (1) AI systems that will write, modify, and reason about code in this project, and (2) future engineers and historians who will examine the architectural decisions that shaped this codebase.
For AI readers: This specification defines the topological constraints you must respect when generating or modifying code. The rules herein are not stylistic preferences—they are mathematical invariants derived from empirical research on cognitive load, transformer attention mechanics, and defect correlation. Violations of these rules will be detected by the SlopChop scanner and will block commits.
For human readers: This document explains why the codebase is structured the way it is. Every constraint is justified by cited research. We have deliberately chosen mathematical rigor over intuition because intuition does not scale, does not transfer between engineers, and cannot be verified by automated tooling.
The Core Thesis
A codebase is not an abstract graph; it is a physical manifold with traversal costs. Every import statement is a cable connecting two coordinates in the directory tree. When related logic is scattered across distant coordinates, we impose a quantifiable tax on both human cognition and AI attention. This document defines the mathematics of that tax and the enforcement mechanisms that eliminate it.

Section 0: Empirical Foundations and Calibration
Before defining any rule, we establish its empirical basis. Every threshold in this specification is drawn from published research or established software metrics literature. This section serves as the citation index for the entire document.
0.1 The Component Metrics Literature
The stability/instability framework used throughout this document originates from Robert C. Martin's component coupling metrics, formalized in his 1994 paper on object-oriented design quality metrics and elaborated in subsequent work on package dependencies.
    • Afferent Coupling (Cₐ): The number of external modules that depend on a given module (fan-in).
    • Efferent Coupling (Cₑ): The number of external modules that a given module depends on (fan-out).
    • Instability Index (I = Cₑ / (Cₐ + Cₑ)): A normalized measure from 0 (maximally stable) to 1 (maximally unstable).
[Source: R.C. Martin, 'OO Design Quality Metrics: An Analysis of Dependencies', 1994; linux.ime.usp.br/~joaomm/mac499/arquivos/referencias/oodmetrics.pdf]
The literature suggests threshold starting points of r ∈ {3, 5} for trunk/leaf classification ratios, and α ∈ [0.2, 0.4] for instability ceilings. Our K ≥ 1.0 threshold corresponds to r ≈ 2.7, which is conservative and within the established range.
0.2 The Cognitive Navigation Literature
Multi-location navigation imposes measurable time penalties on developers.
    • A 2012 Saarland University study comparing physically separated feature representations versus locally bundled code found average task time increases of 12.7%, with specific control-flow tasks showing increases of up to 129%.
    • An Eclipse-based navigation study reported +91 seconds (~15.5%) overhead for tasks involving ~8 disjoint locations, attributed to disorientation and context re-acquisition.
    • A field study of 79 professionals found developers spend 57.62% of time on comprehension activities and 23.96% on navigation.
[Sources: Saarland University FOSD 2012; PPIG 2009 navigation study; Developer activity field study]
Critical finding: There is no universally published breakpoint for 'N directory nodes → X% time increase.' The direction is well-established (multi-location navigation adds ~10-15% overhead on average, with task-specific blowups to ~2×), but exact thresholds require per-repo calibration. Our D ≤ 4 threshold is a conservative default derived from the 'L1/L2 cache' analogy and should be tuned based on empirical observation in this codebase.
0.3 The LLM Attention Degradation Literature
Irrelevant context in LLM prompts causes measurable accuracy degradation. This is the empirical foundation for our 'Sideways Tokens' concept.
Key empirical findings:
    • GSM-IC Study: A model solving math problems at 95.0% accuracy dropped to 72.4% when a single irrelevant sentence was inserted—a 22.6 percentage point collapse from one sentence of noise.
    • Context Dominance: Research analyzing irrelevant-context hallucinations found that for the top-1 candidate answer, the model is context-dominant (driven by irrelevant context) in 23.8% of samples (Llama-3 70B) to 32.4% of samples (Pythia 12B).
    • Demonstration Sensitivity: When a single irrelevant demonstration is prepended, 11.5% of previously correct answers become incorrect.
    • Coherent Distractors: Semantically coherent but irrelevant context (the most dangerous kind—this is exactly what 'Sideways Dependencies' produce) causes ~45% average performance degradation across multiple datasets.
    • Length Alone: Even with distractor tokens masked out of attention, there is still ≥7.9% degradation at 30k masked tokens. With ~30k space tokens, performance drops up to 17-20%.
[Sources: GSM-IC benchmark study; ACL 2025 irrelevant-context hallucination analysis; Contextual Distraction Vulnerability study; arXiv:2510.05381v1 on context length]
0.4 The Defect Correlation Literature
Spatially distant but temporally coupled files correlate with elevated defect rates.
    • Cross-subsystem co-changes result in more bugs than within-subsystem co-changes, with statistically significant larger coefficients (≥95% confidence).
    • Change-coupling measures correlate with bug counts at Spearman ρ > 0.5, peaking above 0.8 in some projects (notably Eclipse).
    • Method-level studies report 'very good positive correlation' between evolutionary coupling and bug-proneness, with buggy methods having significantly more coupling links.
[Sources: GMU ICSE 2013 co-change dispersion study; Kent State MSR 2013 evolutionary coupling study; CASCON 2019 bug-coupling study; ResearchGate fix-inducing changes analysis]
0.5 Threshold Calibration Policy
All thresholds in this document are calibration defaults, not universal constants. They are configured via slopchop.toml and should be tuned based on:
    1. Empirical observation of task completion times in this codebase
    2. Defect correlation analysis against historical bug reports
    3. AI accuracy measurements on codebase-specific tasks
    4. Team feedback on 'false positive' violations

Section 1: The Physicality of Logic and Topological Entropy
1.1 The Central Claim
Modern software engineering treats the repository as an abstract graph where any symbol can reference any other symbol with zero perceived cost. SlopChop v1.2 asserts that this 'Zero-Cost Abstraction of Connection' is empirically false.
A codebase is a physical manifold. Every file occupies a coordinate in a hierarchical tree (the directory structure). Every import statement is a cable connecting two coordinates. When we work on a feature, our brain—and the AI's attention mechanism—must traverse this manifold. The traversal has costs that are measurable and non-zero.
When related logic is scattered across distant coordinates, we create Topological Entropy—a state of disorder where the physical structure of the system actively contradicts its logical purpose.
1.2 The Calculus of Dependency Distance (D)
We formalize the 'distance' between two files by treating the directory tree as a graph and calculating the path length through their Lowest Common Ancestor (LCA).
Let P(x) be the set of directory nodes from the root to file x.
Let LCA(a, b) be the deepest directory node present in both P(a) and P(b).
D(a, b) = (depth(a) - depth(LCA)) + (depth(b) - depth(LCA))
Example Analysis
    • Case A (Local): src/apply/parser.rs → src/apply/types.rs. LCA = src/apply/. D = (3-2) + (3-2) = 2. Result: Low entropy. Logic is bundled.
    • Case B (Warning): src/tui/view.rs → src/apply/parser.rs. LCA = src/. D = (3-1) + (3-1) = 4. Result: Warning threshold. A diagonal cable is forming.
    • Case C (Violation): src/tui/dashboard/widgets/sidebar.rs → src/apply/patch/v1/context_engine.rs. LCA = src/. D = (5-1) + (5-1) = 8. Result: Critical locality violation.
This distance is not just a number; it is a Friction Coefficient. Every integer increase in D represents a contextual leap that must be bridged. The empirical literature (Section 0.2) shows that multi-location navigation adds 10-15% overhead on average, with task-specific blowups to 2× for complex control-flow tasks.
1.3 The Cache Analogy: Spatiotemporal Locality of Reference
The hardware world solved the 'Locality Problem' decades ago. CPUs use L1/L2/L3 caches because physically proximate memory locations are more likely to be accessed together. L1 cache is physically soldered as close to the execution core as possible to minimize signal travel time.
A [FAR] dependency in a codebase is the architectural equivalent of a cache miss. When an AI or developer encounters a dependency with D=8, they must 'fetch from main memory'—leaving their current mental context, navigating to a distant folder, parsing unfamiliar naming conventions, and re-acquiring context.
SlopChop v1.2 enforces Architectural L1/L2 Cache:
    • L1 Distance (D ≤ 2): Always permitted. Local is always safe.
    • L2 Distance (D ≤ 4): Permitted with conditions (target must be a Hub).
    • L3+ Distance (D > 4): Requires explicit Hub status or exemption.
1.4 The Gravity of Depth
Directory depth acts as a gravity well. The deeper a file is buried, the more 'force' is required to reference it from outside its parent. This leads to the Topological Friction Rule:
As the depth of a file increases, its Stability must increase, or its Visibility must decrease.
    • If a file is at src/a/b/c/d/e/f.rs, it is 'heavy.' It should be a private implementation detail that nothing outside of src/a/b/c/d/e/ knows about.
    • If a file is buried that deep but is referenced by src/main.rs, we have a Gravity Leak—reaching into the bottom of a deep well to pull up a bucket.
1.5 Topological Entropy Metric
Topological Entropy (Hₜ) is quantified as the ratio of cross-boundary dependencies to local dependencies:
Hₜ = |{edges where D > 4}| / |{all edges}|
A codebase with Hₜ > 0.3 is in a high-entropy state. It requires constant 'diagonal cabling' to function. This is the formal definition of 'Spaghetti Code'—a system where the physical topology provides no isolation, no containment, and no signal of intent.

Section 2: Component Calculus — The Stability vs. Sensitivity Ratio
2.1 The Problem: Not All Files Are Equal
A file does not exist in isolation; it exists as a node within a directed dependency graph. Without a mathematical classification of a file's role in the network, every file is treated as equal. This is a fallacy.
In any high-integrity system, there must be a clear hierarchy of Responsibility (how many other files depend on this one) and Sensitivity (how many other files this one depends on). Section 2 implements Robert C. Martin's component metrics at the file level to distinguish between the 'Trunks' that support the system and the 'Leaves' that fluctuate with feature requirements.
2.2 Afferent Coupling (Cₐ): Fan-In of Responsibility
Afferent Coupling measures the number of external files that depend on a target file.
Cₐ(x) = |{ y ∈ Repo | y ≠ x ∧ y → x }|
A high Cₐ score indicates that a file is a Dependency Magnet. It has high 'responsibility' because any change to its public API has a blast radius equal to its Cₐ count.
    • The Positive Hub: High Cₐ, low internal complexity. This is a 'Stable Hub'—the cornerstone of the architecture.
    • The Fragile Trunk: High Cₐ, high internal complexity. This is a structural single point of failure.
2.3 Efferent Coupling (Cₑ): Fan-Out of Sensitivity
Efferent Coupling measures the number of other files that a specific file depends upon.
Cₑ(x) = |{ y ∈ Repo | y ≠ x ∧ x → y }|
A high Cₑ score indicates that a file is a Leaf. It is 'sensitive' because it is at the mercy of all its dependencies. If any one of the Cₑ targets changes, the Leaf may require a corresponding update.
The AI Wrangle Connection: An AI working on a file with high Cₑ must hold all N dependencies in its context window. Every unit of Cₑ acts as a multiplier for 'Sideways Tokens.' The empirical literature (Section 0.3) shows that irrelevant context causes 22-45% accuracy degradation. This is why Cₑ matters for AI-native codebases.
2.4 The Instability Index (I)
By intersecting Cₐ and Cₑ, we derive the Instability Index:
I = Cₑ / (Cₐ + Cₑ)    ∈ [0, 1]
    • I → 0 (Maximally Stable): High responsibility, low sensitivity. These are 'Trunk Lines'—the bedrock. Changing them is expensive, so they change rarely.
    • I → 1 (Maximally Unstable): Low responsibility, high sensitivity. These are 'Leaf Features'—the volatile outer shell. Changing them is cheap because nothing depends on them.
The Topological Integrity Goal: Dependencies must point toward stability. A stable file (I ≈ 0) should never depend on an unstable file (I ≈ 1). If src/types.rs (I=0.05) imports src/tui/sidebar_view.rs (I=0.95), the architecture is inverted.
2.5 The Skew Score (K): Logarithmic Normalization
The Instability Index fails to capture magnitude. A file with Cₐ=1, Cₑ=0 and a file with Cₐ=100, Cₑ=0 both have I=0, but their 'trunk-ness' is vastly different.
SlopChop implements the Skew Score (K) using a natural logarithm of the smoothed coupling ratio:
K = ln((Cₐ + 1) / (Cₑ + 1))
    • K > 0: Trunk-leaning (more files depend on this than it depends on).
    • K < 0: Leaf-leaning (this depends on more files than depend on it).
    • |K|: How strongly it leans.
The '+1' is a Laplace Correction (smoothing). It prevents division-by-zero errors and acts as a damping factor for small files.
Examples
    • Tiny utility: Cₐ=1, Cₑ=0 → K = ln(2/1) = 0.69. Leans toward Hub, but not a core hub.
    • Major types file: Cₐ=50, Cₑ=2 → K = ln(51/3) = 2.83. Confirmed high-integrity Trunk.
    • Feature orchestrator: Cₐ=1, Cₑ=12 → K = ln(2/13) = -1.87. Strong Leaf signal.
2.6 The Taxonomy of Node Identities
Using K and raw coupling counts, SlopChop classifies every file into one of four Topological Quadrants:
Quadrant
Criteria
Policy
Stable Hub
K ≥ 1.0, Cₐ ≥ 5
Allowed to have high D. These are the vertical trunks.
Volatile Leaf
K < 0, Cₑ ≥ 5
Must have D ≤ 4. These are horizontal bundles.
Isolated Deadwood
Cₐ + Cₑ < 2
Audit flag. Ghost logic—often abandoned features.
God Module
Cₐ > 20, Cₑ > 20
Danger zone. Flag for immediate modular fission.

2.7 The Distance-Skew Intersection: The Universal Locality Algorithm
The power of Section 2 lies in how we use K to validate D from Section 1. We no longer punish all long-distance dependencies; we only punish Spaghetti Cables.
For every dependency edge A → B:
    1. Calculate D(A, B).
    2. If D(A, B) ≤ 2 (L1 Cache): PASS. Local is always safe.
    3. If D(A, B) > 2: Retrieve Skew Score K for target B.
• If B.K ≥ hub_threshold (default: 1.0): PASS. Connecting to a Hub from afar is 'Vertical Routing'.
• If B.K < hub_threshold: FAIL. Connecting to a Leaf from afar is 'Sideways Spaghetti'.
This logic ensures that feature-to-feature communication is strictly local, while feature-to-hub communication is globally permissible. It forces developers to extract shared logic into a Hub if they want to use it across different modules.

Section 3: The Cognitive Tax — Information Foraging and Navigation Disorientation
3.1 Code as Cognitive Scaffolding
The directory structure of a repository is the primary user interface for the developer's brain. It acts as cognitive scaffolding—a map that allows the human mind to offload complexity onto the environment. When the map is clear, the mind is free to solve logic problems. When the map is 'Slop,' the mind consumes its finite energy budget simply trying to find the logic.
3.2 Information Foraging Theory
Information Foraging Theory (IFT), pioneered by Pirolli and Card at Xerox PARC, posits that humans navigate information environments using the same evolutionary strategies animals use to forage for food. Developers maximize 'Information Gain' while minimizing 'Interaction Cost.'
The 'Information Scent'
The 'Scent' is the collection of proximal cues—folder names, file names, sibling files—that suggest a path toward the goal.
    • High Scent: I'm looking for 'Surgical Patching' logic. I see src/apply/. I enter. I see src/apply/patch/. The scent is strong. My brain is in 'Exploitation Mode.'
    • Decayed Scent: I'm in src/apply/patch/. I need to understand the visual diff. But that logic is in src/tui/dashboard/widgets/diff_view.rs. The scent has evaporated. My brain is forced into 'Exploratory Mode.'
3.3 Quantified Navigation Costs
The empirical literature (Section 0.2) provides concrete numbers:
    • Average overhead from physically separated features: 12.7% increased task time.
    • Control-flow tasks across distant files: Up to 129% increased completion time.
    • Navigation/disorientation cost per major context switch: +91 seconds average.
    • Developer time spent on navigation vs. creation: 23.96% of all work time.
These are not theoretical concerns. They are measured realities. The D ≤ 4 threshold is calibrated to prevent the worst-case 'Control-Flow Blowup' scenario, where task completion time more than doubles.
3.4 Working Memory and the Magic Number
George Miller's 'The Magical Number Seven, Plus or Minus Two' establishes the limit of human working memory. Humans manage complexity through 'Chunking'—grouping related items into single mental units.
    • The Neat Rack: src/apply/ is one 'Chunk.' Everything needed to understand 'Apply' is inside it. The entire subsystem fits in 1-2 slots of working memory.
    • The Spaghetti Pole: To understand 'Apply,' I need src/apply/, but also src/tui/, src/audit/, and src/common/types/. Each distant dependency consumes a working memory slot. If a file has 8 distant dependencies, the brain is at capacity before reading the first line of code.
When logic is physically local, the 'Peripheral Context' acts as a physical reminder of purpose. If I see parser.rs next to lexer.rs, my brain automatically chunks them. If parser.rs is alone and lexer.rs is D=6 away, the chunking mechanism fails. This is Topological Interference—the physical layout actively sabotaging the brain's ability to compress information.
3.5 Progressive Disclosure: The Rule of 10
Every file in a directory is a 'Choice' that the brain must process. Following Hick's Law, the more choices at a single level, the longer the 'Scan Cost.'
    • Violation: A src/ folder with 40 files. Scan cost is overwhelming. The brain stops reading and starts pattern matching, leading to missed critical files.
    • Solution: A src/ folder with 7 sub-folders. The brain makes one quick decision (choose 1 of 7) and moves deeper into focused context.
SlopChop v1.2 implements: Any directory with > 10 peer files triggers a Categorization Violation. 10 is the upper limit of the Magic Number plus margin. This ensures the codebase is always organized into manageable, high-scent bundles.
3.6 The Flat vs. Deep Tradeoff
Both extremes are problematic:
    • Too Flat: Creates 'Choice Paralysis.' Users must scan 50 items instead of choosing 1 of 5 categories. Critical features look the same as niche ones.
    • Too Deep: Creates 'Getting Lost.' Users can't build mental models when they must navigate 7+ levels to find anything.
The solution is Shallow-But-Grouped: keep depth low (2-4 levels), but don't flatten so much that folders become dumping grounds. SlopChop enforces this through combined D-limit and categorization-limit checks.

Section 4: Attention Economics — LLM Mechanics and Prompt Pollution
4.1 The Transformer as Co-Processor
The Large Language Model is no longer a 'tool' external to development; it is a secondary processor integrated into the codebase's execution flow. If the codebase topology is a 'Spaghetti Pole,' the AI's internal attention weights are pulled in conflicting directions.
This section translates the empirical findings from Section 0.3 into architectural rules. We do not invent new formulas—we apply measured phenomena to codebase design.
4.2 The Empirical Case Against Sideways Tokens
A Sideways Token is any token included in a context window that is semantically irrelevant to the current task but required due to distant, non-Hub dependencies.
The empirical literature establishes the damage:
Finding
Implication for Codebase Design
1 irrelevant sentence → 22.6 point accuracy drop (GSM-IC)
Even small amounts of noise cause catastrophic reasoning failures.
24-32% of answers are context-dominant (driven by irrelevant context)
Roughly 1 in 4 AI suggestions may be contaminated by Sideways context.
Coherent distractors → ~45% average degradation
Sideways Dependencies are 'coherent distractors'—plausibly related, actually harmful.
Context length alone → 7.9-20% degradation
Minimize total context size, not just 'bad' context.

4.3 Why the Law of Locality Is a Transformer Optimization
When a codebase has high Topological Entropy, the slopchop pack --focus command is forced to pull in files from across the tree. This creates fragmented prompts where critical dependencies are buried in what the 'Lost in the Middle' research calls the 'Attention Dead Zone.'
The Law of Locality (D ≤ 4) ensures that dependencies are logically and physically clustered. This allows SlopChop to pack them in a way that keeps 'Signal' at the attention peaks of the prompt—the beginning and end, where LLM performance is highest.
4.4 The Foveal/Peripheral Contextual Model
SlopChop's context generation mirrors the human eye's foveal and peripheral vision:
    1. The Fovea (High Detail): The target file and its immediate neighbors (D ≤ 2). Provided in full source.
    2. The Periphery (Signatures): Stable Hubs (K > 1) and distant dependencies (D > 4). Provided as docstring skeletons.
By using skeletons for distant Hubs, we exploit the LLM's ability to use summary information to satisfy symbolic references without filling the context with irrelevant implementation noise.
4.5 Symbol Disambiguation and Namespace Isolation
Spaghetti codebases suffer from Symbolic Overlap. If every folder has its own utils.rs or types.rs, and they're all linked via distant dependencies, the AI's attention mechanism can confuse symbols.
In a high-locality system:
    • Related symbols are physically isolated in their feature bundles.
    • When the AI sees a reference to Error, it doesn't choose between 5 different Error definitions. The 'correct' definition is physically nearest to the Query.
The Law of Locality acts as a Namespace Linter. By blocking sideways imports, we ensure every symbol in the AI's context window has a unique, local 'Scent.'
4.6 Expected Impact
Based on the empirical literature, we expect the following impacts from enforcing the Law of Locality:
    • Reduction in AI reasoning errors on codebase-specific tasks: 15-25% (based on distractor removal studies).
    • Reduction in 'AI Wrangle' incidents (AI suggests architecturally inappropriate code): 30-50% (based on context-dominant frequency reduction).
    • Reduction in prompt token spend for equivalent tasks: 20-40% (based on Sideways Token elimination).
Note: These are expected ranges based on published research. Actual impact should be measured empirically for this codebase and reported in subsequent revisions of this document.

Section 5: The Locality Matrix — A Four-Pass Graph Algorithm
5.1 From Theory to Binary
Sections 1-4 established the justifications for the Law of Locality. Section 5 defines the implementation—a multi-pass graph traversal engine that evaluates the repository as a holistic, directed manifold.
The engineering challenge is performance and precision. Evaluating per-file complexity is O(n); evaluating the entire topological matrix of a repository with thousands of files requires an optimized multi-pass approach.
5.2 Pass 1: Global Trace (Static Graph Extraction)
The first pass extracts every logical connection (edge) within the system.
Implementation
    • Iterate through every file using Tree-Sitter AST parsing.
    • Execute language-specific QueryKind::Imports patterns.
    • Map every import to a canonical, repo-root-relative PathBuf via resolver.rs.
    • Store results in a TopologicalDependencyMap (adjacency list): HashMap<PathBuf, Vec<PathBuf>>.
    • Execute in parallel using rayon for sub-second performance on large repos.
5.3 Pass 2: Calculus Pass (Stability Scoring)
The second pass computes the 'identity' of every file.
Implementation
    • For every node N: count outgoing edges for Cₑ, perform reverse-lookup for Cₐ.
    • Apply Skew formula: K = ln((Cₐ + 1) / (Cₑ + 1)).
    • Cache K values in a thread-safe DashMap.
    • Optionally ignore tests/ directories to prevent test-only dependencies from inflating Hub Status.
5.4 Pass 3: Euclidean Pass (Spatial Coordinate Mapping)
The third pass maps logical edges onto the physical directory tree.
Implementation
    • For every edge A → B: find LCA via path-component matching.
    • Compute D = (depth(A) - depth(LCA)) + (depth(B) - depth(LCA)).
    • Classify edge geometry: Vertical (LCA is immediate parent) vs. Sideways (LCA is distant ancestor).
    • Memoize LCA results—if multiple files in src/tui/ depend on src/types.rs, calculate once.
5.5 Pass 4: Judgment Pass (Topological Integrity Gate)
The fourth pass applies policy to the computed metrics.
The Logic Gate
For every edge A → B:
    1. Filter 1 (Distance): If D(A,B) ≤ max_distance (default: 4) → PASS
    2. Filter 2 (Hub Status): If B.K ≥ hub_threshold (default: 1.0) → PASS
    3. Filter 3 (Exemption): If B matches rules.locality.hubs (manual overrides) → PASS
    4. REJECTION: If no filters match → Locality Violation
Mode Control
    • mode = 'error': Any violation returns non-zero exit code, blocking slopchop apply --promote.
    • mode = 'warn': Violations printed with [FAR] marker, but transaction proceeds.
5.6 Parallel Pass: Categorization Enforcement
Simultaneously with edge analysis, SlopChop sweeps directory nodes for Hick's Law compliance.
    • For every directory: count immediate child files.
    • If count > categorization_limit (default: 10): emit Categorization Violation.
    • Advice: 'Directory contains N files. This exceeds the Magic Number threshold. Group into sub-directories.'
5.7 Performance Guarantees
    • No Cloning: String interning and path-references prevent heap bloat.
    • Memoization: LCA results cached between passes.
    • Deterministic Ordering: Output sorted alphabetically for reproducible reports.
    • Parallelization: Rayon-based parallel iteration for I/O and CPU-bound passes.

Section 6: Structural Restoration — Co-Change Gravity and Refactoring Heuristics
6.1 The Fourth Dimension: Temporal Locality
Sections 1-5 define Topological Integrity as a relationship between static dependencies and directory structure. This provides a 'snapshot' of integrity. However, software evolves through time.
Temporal Locality asserts that files which consistently change together are logically part of the same 'Bundle,' regardless of their current physical location. If src/apply/parser.rs and src/tui/view.rs are modified in the same commit 90% of the time, they possess high Co-Change Confidence. If their D is also high, the repository is in a state of Topological Discordance—the physical structure is lying about the logical reality of the work.
6.2 Empirical Basis: Co-Change and Defects
The literature (Section 0.4) establishes:
    • Cross-subsystem co-changes result in more bugs than within-subsystem co-changes (statistically significant, ≥95% confidence).
    • Change-coupling correlates with bug counts at ρ = 0.5-0.8.
    • Spatially distant + temporally coupled = elevated regression-risk hotspot.
SlopChop v1.2 uses git history to detect these patterns and flag them for remediation.
6.3 The Discordance Score (Γ)
We define the gap between temporal necessity and spatial reality:
Γ = CoChangeConfidence(a, b) × D(a, b)
    • Low Γ: Files that change together live together. The rack is neat.
    • High Γ: Files that change together are far apart. This is a 'Strained Cable.'
6.4 Automated Refactoring Heuristics
When the scanner identifies a high-entropy Sideways Dependency, it applies architectural heuristics:
The 'Bundle' Suggestion (Locality Hoisting)
If File A depends on Leaf File B, D > 4, and both files have low Cₐ (they're private implementation details):
Suggestion: 'These files are a private couple living in separate houses. Move File B into File A's directory, or create a new feature directory src/combined_feature/ for both.'
Result: D drops from 8 to 1. Information Scent is restored.
The 'Bridge' Suggestion (Abstraction Hoisting)
If File A depends on Leaf File B, but File B has growing Cₐ > 3 (it's becoming a Hub):
Suggestion: 'File B is becoming a popular destination. It is too volatile (K < 0) to be a Hub. Extract the shared types/traits from File B into a new Stable Hub in src/core/ or src/common/ (K > 1).'
Result: File A now depends on a Stable Hub. Distance checks are exempted.
6.5 The Ideal Tree Projection
SlopChop's map --ideal flag shows a 'holographic projection' of where files should be to minimize cognitive load. The ideal tree is one where:
    • Every folder contains 7 ± 2 items (Hick's Law optimization).
    • Total System Distance (ΣD) is minimized.
    • Average Skew (K̄) is maximized (System Stability).
This projection uses clustering on the dependency graph to suggest mathematically optimal directory structure.
6.6 Scope Note
Section 6 capabilities (git history analysis, automated refactoring suggestions, ideal tree projection) are v1.3 scope. The v1.2 release focuses on detection and enforcement (Sections 1-5). Section 6 is documented here as the architectural direction for future work.

Section 7: Edge Cases, Exemptions, and Escape Hatches
7.1 Legitimate Exceptions
No heuristic is perfect. The Law of Locality will sometimes flag code that is, in fact, correctly placed. This section defines the mechanisms for handling exceptions.
7.2 Manual Hub Declaration
Files can be explicitly declared as Hubs in slopchop.toml:
[rules.locality]
hubs = [
  "src/types.rs",
  "src/errors.rs",
  "src/config.rs",
]
Files in this list are granted Hub status regardless of their computed K score. Use sparingly—this is an override, not a blanket exemption.
7.3 Path Pattern Exemptions
Entire path patterns can be exempted from locality checks:
[rules.locality]
exempt_patterns = [
  "src/generated/*",
  "src/vendor/*",
  "src/proto/*",
]
Use for: generated code, vendored dependencies, protocol buffers, and other machine-generated files that cannot be relocated.
7.4 Test File Handling
By default, tests/ directories are excluded from Hub status computation to prevent test-only dependencies from inflating a file's responsibility score.
Test → Production dependencies are still checked, but with relaxed thresholds:
[rules.locality]
test_max_distance = 6  # More lenient for tests
test_mode = "warn"    # Never block on test violations
7.5 Circular Dependency Handling
The graph traversal detects cycles during Pass 1. When a cycle is detected:
    1. The cycle is flagged as a Structural Violation (separate from Locality Violations).
    2. Cycle participants are marked with artificially high Cₐ and Cₑ (both set to cycle size).
    3. This triggers 'God Module' classification, forcing a fission recommendation.
7.6 Monorepo Boundaries
For monorepos with multiple logical projects, use workspace boundaries:
[workspaces]
roots = [
  "packages/frontend",
  "packages/backend",
  "packages/shared",
]
Dependencies within a workspace root are measured normally. Dependencies across workspace roots must target files in packages/shared (which is implicitly granted Hub status) or be explicitly exempted.
7.7 Inline Exemption Comments
For one-off exceptions, use inline comments:
// slopchop:allow-far(reason: legacy integration)
use crate::legacy::old_module::OldType;
The scanner will skip this import for locality checks but will record it in the audit log. Inline exemptions should be rare and always include a reason.
7.8 Gradual Adoption: The Baseline
For legacy codebases, SlopChop supports a baseline model:
slopchop scan --generate-baseline > .slopchop-baseline.json
Once a baseline is generated, subsequent scans only report new violations. This allows gradual adoption: fix violations in new code, address legacy violations over time.

Section 8: Configuration Reference
8.1 Complete slopchop.toml Schema
[rules.locality]
# Core thresholds (calibration defaults)
max_distance = 4           # Maximum D for non-Hub dependencies
hub_threshold = 1.0        # Minimum K to qualify as Hub
categorization_limit = 10  # Maximum files per directory

# Node classification thresholds
min_hub_afferent = 5       # Minimum Cₐ for Hub status
god_module_threshold = 20  # Cₐ AND Cₑ above this = God Module
deadwood_threshold = 2     # Cₐ + Cₑ below this = Deadwood

# Enforcement mode
mode = "error"             # "error" | "warn" | "off"

# Manual overrides
hubs = []                  # Paths with explicit Hub status
exempt_patterns = []       # Glob patterns to skip

# Test handling
test_max_distance = 6
test_mode = "warn"
exclude_tests_from_hub_calc = true

[workspaces]
roots = []                 # Monorepo workspace boundaries

[baseline]
path = ".slopchop-baseline.json"
enabled = false
8.2 CLI Reference
# Scan for violations
slopchop scan --locality

# Generate topology map
slopchop map --show-distance --show-skew

# Show ideal tree projection (v1.3)
slopchop map --ideal

# Generate baseline for gradual adoption
slopchop scan --generate-baseline > .slopchop-baseline.json

# Pack context for AI with locality optimization
slopchop pack --focus src/apply/mod.rs --foveal-depth 2

# Audit with refactoring suggestions (v1.3)
slopchop audit --locality --suggest

Section 9: Summary — The Calculus of the Neat Image
High-integrity systems are not built by 'cleaning up' code; they are built by enforcing the physics of logic.
What We Established
    • Section 0 grounded every threshold in published research, establishing that our rules are derived from evidence, not intuition.
    • Section 1 defined the Manifold—the physical space of the codebase and the Dependency Distance metric.
    • Section 2 defined the Calculus—the Stability vs. Sensitivity ratio that distinguishes Trunks from Leaves.
    • Section 3 defined the Biology—how the human brain forages for information and why physical layout matters for cognition.
    • Section 4 defined the Attention—how LLM transformers process context and why Sideways Tokens cause measurable accuracy degradation.
    • Section 5 defined the Algorithm—the four-pass graph traversal that implements these rules in code.
    • Section 6 defined the Restoration—how to fix violations and where the architecture should evolve.
    • Section 7 defined the Escape Hatches—how to handle legitimate exceptions without undermining the system.
    • Section 8 provided the Configuration—the complete reference for tuning and operating SlopChop.
The Universal Locality Algorithm (Final Statement)
For every dependency edge A → B in this codebase:
    3. If D(A, B) ≤ 2: PASS. Local is always safe.
    4. If D(A, B) > 2 and B.K ≥ 1.0: PASS. Vertical routing to a Hub is permitted.
    5. If D(A, B) > 2 and B.K < 1.0: FAIL. Sideways Spaghetti. Refactor or exempt.
This algorithm ensures that feature-to-feature communication is strictly local, while feature-to-hub communication is globally permissible. It enforces a Stability Gradient where information flows from unstable, concrete outer rings toward stable, abstract inner trunks.
For AI Readers
When you generate or modify code in this codebase, respect these invariants:
    • Do not create new dependencies with D > 4 unless the target is a Hub (K ≥ 1.0).
    • Do not create files that would exceed the 10-file-per-directory limit without proposing a subdirectory structure.
    • When you need shared functionality, check if an existing Hub provides it before creating new cross-module dependencies.
    • If you must create a far dependency, document why in a comment and consider whether the target should be promoted to Hub status.
For Future Readers
This document exists because we believed that architectural discipline should be specified, not just aspired to. We chose mathematics over intuition because intuition doesn't scale, doesn't transfer, and can't be verified by machines.
Every rule in this document can be questioned. Every threshold can be recalibrated. But the underlying principle—that logic has physicality and traversal has cost—is grounded in decades of research on program comprehension, cognitive load, and now, transformer attention mechanics.
If you find a rule that is wrong, fix it. But document your reasoning with the same rigor we have attempted here. The goal is not to be right; the goal is to be testably right, so that future evidence can correct us.

SlopChop v1.2 doesn't just clean the slop—it optimizes the codebase for the speed of thought, both biological and silicon.

End of Specification.
