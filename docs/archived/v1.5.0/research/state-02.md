# SOTA Report: State Ownership Spread Metrics (State-02)

**Date**: January 2026  
**Subject**: Metrics beyond basic coupling for quantifying state distribution, encapsulation, and ownership fragmentation.

## 1. Executive Summary
Measuring how state "leaks" or "spreads" across a codebase requires metrics that capture encapsulate effectiveness (MOOD suite), internal cohesion (LCOM), and the socio-technical distribution of responsibility (Author Entropy). This report provides formal definitions and mathematical formulas for the leading software engineering metrics used in State-of-the-Art (SOTA) defect prediction and architectural analysis.

---

## 2. Encapsulation Metrics: The MOOD Suite
The *Metrics for Object-Oriented Design* (MOOD) suite provides high-level indicators of how well a system hides its internal state.

### 2.1 Attribute Hiding Factor (AHF)
AHF measures the average "invisibility" of attributes across all classes.

*   **Formula**:  
    \[ AHF = \frac{\sum_{i=1}^{TC} (1 - V(A_i))}{Total Attributes} \]
    Where \( V(A_i) \) is the visibility of attribute \( A_i \), defined as the fraction of other classes that can access it.
*   **Interpretation**: An AHF of 100% means all attributes are `private`. SOTA research suggest that a low AHF is a primary leading indicator of "State Leakage," significantly increasing the probability of side-effect bugs during refactoring.
*   **Citation**: Brito e Abreu, F. (1995). *The MOOD Metrics Set*. Workshop on Information Systems and Case.

---

## 3. Cohesion & Fragmentation Metrics
These metrics analyze the internal structure of a class to see if it is managing too many independent states.

### 3.1 LCOM1 (Chidamber & Kemerer)
The original "Lack of Cohesion in Methods" metric.

*   **Formula**:  
    Let \( P \) be the number of method pairs that share *no* instance variables.  
    Let \( Q \) be the number of method pairs that *do* share instance variables.  
    \[ LCOM1 = (P > Q) ? (P - Q) : 0 \]
*   **Citation**: Chidamber, S. R., & Kemerer, C. F. (1994). *A Metrics Suite for Object Oriented Design*. IEEE Trans. Softw. Eng.

### 3.2 LCOM4 (Hitz & Montazeri)
A more robust SOTA variant using graph theory.

*   **Logic**: A class is modeled as an undirected graph where methods are nodes. An edge exists between methods if they share an attribute or if one calls the other.
*   **Metric**: LCOM4 is the number of **connected components** in this graph.
*   **Interpretation**:  
    *   **LCOM4 = 1**: Ideal (Unified state ownership).
    *   **LCOM4 > 1**: Fragmented (The class is actually 2 or more independent entities and should be split).
*   **Citation**: Hitz, M., & Montazeri, B. (1995). *Measuring Coupling and Cohesion in Object-Oriented Systems*.

---

## 4. Socio-Technical Ownership: Author Entropy
Traditional metrics ignore *who* modifies the state. SOTA research uses Information Theory to measure the "diffusion" of state ownership among developers.

*   **Mechanism**: **Author Entropy (Shannon Entropy)**  
    \[ H = -\sum p_i \log_2 p_i \]
    Where \( p_i \) is the proportion of changes (or lines of state-modifying code) contributed by developer \( i \) to a specific module.
*   **SOTA Insight**: High entropy (imbalanced contributions or "too many cooks") in state-heavy modules is a strong predictor of "Defect Density." It quantifies the lack of a "Primary Owner" for the state.
*   **Citation**: D'Ambros, M., et al. (2010). *An extensive comparison of bug prediction metrics*. MSR '10.

---

## 5. Defect Prediction: Module Severity Density (MSD)
MSD moves beyond binary "bug/no-bug" classification to measure the *impact* of state failures.

*   **Formula**:  
    \[ MSD = \frac{\sum (Severity \times Defects)}{Size (e.g., kLOC)} \]
*   **Significance**: High MSD in specific clusters identifies "Mutation Hotspots"—parts of the system where state management is both complex and high-risk.
*   **Citation**: Hong, E. (2000). *Software fault-proneness prediction using MSD*.

---

## 6. Metric Comparison Table

| Metric | Dimension | Goal | Threshold (Red Flag) |
| :--- | :--- | :--- | :--- |
| **AHF** | Encapsulation | Prevent state leakage | < 60% |
| **LCOM4** | Cohesion | Prevent "God Object" fragmentation | > 1 |
| **Entropy** | Ownership | Identify ownership vacuums | > 0.8 (Imbalance) |
| **MSD** | Risk | Prioritize state refactoring | Top 10% of modules |
