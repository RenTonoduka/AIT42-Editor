//! Prompt construction for LLM-based complexity estimation

/// Builder for constructing complexity estimation prompts
pub struct PromptBuilder {
    task_description: String,
    current_subtasks: usize,
    additional_context: Option<String>,
}

impl PromptBuilder {
    /// Create a new prompt builder
    pub fn new() -> Self {
        Self {
            task_description: String::new(),
            current_subtasks: 0,
            additional_context: None,
        }
    }

    /// Set the task description to analyze
    #[must_use]
    pub fn with_task(mut self, description: &str) -> Self {
        self.task_description = description.to_string();
        self
    }

    /// Set the current number of subtasks (if task is being re-analyzed)
    #[must_use]
    pub fn with_current_subtasks(mut self, count: usize) -> Self {
        self.current_subtasks = count;
        self
    }

    /// Add additional context for the LLM
    #[must_use]
    pub fn with_context(mut self, context: &str) -> Self {
        self.additional_context = Some(context.to_string());
        self
    }

    /// Build the final prompt string
    pub fn build(&self) -> String {
        let context_section = if let Some(ctx) = &self.additional_context {
            format!(
                "\nAdditional Context:\n{}\n",
                ctx
            )
        } else {
            String::new()
        };

        let subtask_hint = if self.current_subtasks > 0 {
            format!(
                "\nNote: This task currently has {} subtasks. Consider if this needs adjustment.\n",
                self.current_subtasks
            )
        } else {
            String::new()
        };

        format!(
            r#"Analyze this software development task and classify its Big-Omega complexity.

Task: "{}"{}{}
Classify using one of these categories EXACTLY as shown:
- Ω(1) - Constant: Simple config changes, documentation updates, trivial variable assignments
- Ω(log n) - Logarithmic: Binary search implementations, balanced tree operations, divide-and-conquer
- Ω(n) - Linear: Standard CRUD operations, single-pass array processing, simple API endpoints
- Ω(n log n) - Linearithmic: Sorting algorithms, index building, merge operations, query optimization
- Ω(n²) - Quadratic: Matrix operations, nested loops, complex state management, graph algorithms
- Ω(2^n) - Exponential: Combinatorial problems, brute-force search, recursive backtracking

Consider these factors:
1. Implementation difficulty (code complexity, edge cases)
2. Testing requirements (unit tests, integration tests, edge case coverage)
3. Integration complexity (API surface area, dependency management)
4. Debugging overhead (error handling, logging, observability)
5. Documentation needs (API docs, examples, tutorials)

Return ONLY a valid JSON object with NO markdown formatting, NO code blocks, NO explanations:
{{
  "complexity_class": "Ω(n)",
  "reasoning": "Brief 1-2 sentence explanation of why this classification was chosen",
  "recommended_subtasks": 4,
  "confidence": 0.85
}}

IMPORTANT:
- Use EXACT complexity class strings from the list above (including the Ω symbol)
- recommended_subtasks must be between 1 and 20
- confidence must be between 0.0 and 1.0
- Return ONLY the JSON object, nothing else"#,
            self.task_description, context_section, subtask_hint
        )
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_prompt() {
        let prompt = PromptBuilder::new()
            .with_task("Implement user authentication")
            .build();

        assert!(prompt.contains("Implement user authentication"));
        assert!(prompt.contains("Ω(1)"));
        assert!(prompt.contains("Ω(n)"));
        assert!(prompt.contains("Ω(2^n)"));
        assert!(prompt.contains("Return ONLY a valid JSON object"));
    }

    #[test]
    fn test_prompt_with_subtasks() {
        let prompt = PromptBuilder::new()
            .with_task("Optimize database queries")
            .with_current_subtasks(3)
            .build();

        assert!(prompt.contains("Optimize database queries"));
        assert!(prompt.contains("currently has 3 subtasks"));
    }

    #[test]
    fn test_prompt_with_context() {
        let prompt = PromptBuilder::new()
            .with_task("Add API endpoint")
            .with_context("Legacy system with complex authentication")
            .build();

        assert!(prompt.contains("Add API endpoint"));
        assert!(prompt.contains("Additional Context"));
        assert!(prompt.contains("Legacy system with complex authentication"));
    }

    #[test]
    fn test_prompt_with_all_options() {
        let prompt = PromptBuilder::new()
            .with_task("Refactor authentication module")
            .with_current_subtasks(5)
            .with_context("High security requirements")
            .build();

        assert!(prompt.contains("Refactor authentication module"));
        assert!(prompt.contains("currently has 5 subtasks"));
        assert!(prompt.contains("Additional Context"));
        assert!(prompt.contains("High security requirements"));
    }

    #[test]
    fn test_prompt_contains_critical_instructions() {
        let prompt = PromptBuilder::new()
            .with_task("Test task")
            .build();

        // Ensure critical instructions are present
        assert!(prompt.contains("Return ONLY a valid JSON object"));
        assert!(prompt.contains("NO markdown formatting"));
        assert!(prompt.contains("recommended_subtasks must be between 1 and 20"));
        assert!(prompt.contains("confidence must be between 0.0 and 1.0"));
    }

    #[test]
    fn test_default_builder() {
        let builder = PromptBuilder::default();
        assert_eq!(builder.task_description, "");
        assert_eq!(builder.current_subtasks, 0);
        assert!(builder.additional_context.is_none());
    }
}
