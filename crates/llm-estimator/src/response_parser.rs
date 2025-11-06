//! Response parsing for LLM complexity estimates

use crate::error::ParseError;
use omega_theory::ComplexityClass;
use serde::{Deserialize, Serialize};

/// Complexity estimate returned by the LLM
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexityEstimate {
    /// The Big Omega complexity class (e.g., "Ω(n)")
    pub complexity_class: String,

    /// Reasoning for the classification
    pub reasoning: String,

    /// Recommended number of subtasks
    pub recommended_subtasks: usize,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,
}

impl ComplexityEstimate {
    /// Parse the complexity class string into a ComplexityClass enum
    ///
    /// # Errors
    ///
    /// Returns `ParseError::InvalidComplexityClass` if the string doesn't match any known class
    pub fn to_complexity_class(&self) -> std::result::Result<ComplexityClass, ParseError> {
        match self.complexity_class.as_str() {
            "Ω(1)" => Ok(ComplexityClass::Constant),
            "Ω(log n)" => Ok(ComplexityClass::Logarithmic),
            "Ω(n)" => Ok(ComplexityClass::Linear),
            "Ω(n log n)" => Ok(ComplexityClass::Linearithmic),
            "Ω(n²)" => Ok(ComplexityClass::Quadratic),
            "Ω(2^n)" => Ok(ComplexityClass::Exponential),
            other => Err(ParseError::InvalidComplexityClass(other.to_string())),
        }
    }

    /// Validate the estimate's fields
    ///
    /// # Errors
    ///
    /// Returns appropriate `ParseError` if validation fails
    pub fn validate(&self) -> std::result::Result<(), ParseError> {
        // Validate complexity class
        self.to_complexity_class()?;

        // Validate subtask count
        if self.recommended_subtasks == 0 || self.recommended_subtasks > 20 {
            return Err(ParseError::InvalidSubtaskCount(self.recommended_subtasks));
        }

        // Validate confidence score
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(ParseError::InvalidConfidence(self.confidence));
        }

        Ok(())
    }
}

/// Parser for LLM responses
pub struct ResponseParser;

impl ResponseParser {
    /// Parse a JSON string from the LLM into a ComplexityEstimate
    ///
    /// # Arguments
    ///
    /// * `json_str` - The JSON string returned by the LLM
    ///
    /// # Returns
    ///
    /// A validated `ComplexityEstimate`
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - JSON parsing fails
    /// - Required fields are missing
    /// - Field values are invalid
    pub fn parse(json_str: &str) -> std::result::Result<ComplexityEstimate, ParseError> {
        // Try to extract JSON from potential markdown code blocks
        let cleaned = Self::extract_json(json_str);

        // Parse JSON
        let estimate: ComplexityEstimate = serde_json::from_str(&cleaned).map_err(|e| {
            tracing::warn!("JSON parsing failed: {}", e);
            ParseError::InvalidJson(e.to_string())
        })?;

        // Validate all fields
        estimate.validate()?;

        Ok(estimate)
    }

    /// Extract JSON from potential markdown code blocks or explanatory text
    ///
    /// LLMs sometimes wrap JSON in markdown or add explanations. This function
    /// attempts to extract just the JSON object.
    fn extract_json(text: &str) -> String {
        let trimmed = text.trim();

        // Check for markdown code blocks with language specifier
        if let Some(json_start) = trimmed.find("```json") {
            // Find the closing ``` after the opening ```json
            let search_from = json_start + 7; // After ```json
            if let Some(relative_end) = trimmed[search_from..].find("```") {
                let start = search_from; //  Right after ```json
                let end = search_from + relative_end; // Absolute position of closing ```
                return trimmed[start..end].trim().to_string();
            }
        }

        // Check for generic code blocks (without language specifier)
        if let Some(json_start) = trimmed.find("```") {
            let search_from = json_start + 3; // After first ```
            if let Some(relative_end) = trimmed[search_from..].find("```") {
                let start = search_from;
                let end = search_from + relative_end;
                return trimmed[start..end].trim().to_string();
            }
        }

        // Try to find JSON object boundaries
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                if start < end {
                    return trimmed[start..=end].to_string();
                }
            }
        }

        // Return as-is if no patterns match
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{
            "complexity_class": "Ω(n)",
            "reasoning": "Standard CRUD operation",
            "recommended_subtasks": 4,
            "confidence": 0.85
        }"#;

        let estimate = ResponseParser::parse(json).unwrap();
        assert_eq!(estimate.complexity_class, "Ω(n)");
        assert_eq!(estimate.reasoning, "Standard CRUD operation");
        assert_eq!(estimate.recommended_subtasks, 4);
        assert_eq!(estimate.confidence, 0.85);
    }

    #[test]
    fn test_parse_with_markdown() {
        let json = r#"```json
{
    "complexity_class": "Ω(1)",
    "reasoning": "Simple config change",
    "recommended_subtasks": 1,
    "confidence": 0.95
}
```"#;

        let estimate = ResponseParser::parse(json).unwrap();
        assert_eq!(estimate.complexity_class, "Ω(1)");
    }

    #[test]
    fn test_parse_with_explanation() {
        let json = r#"Here's my analysis:

{
    "complexity_class": "Ω(n²)",
    "reasoning": "Nested loops required",
    "recommended_subtasks": 7,
    "confidence": 0.75
}

This should work well."#;

        let estimate = ResponseParser::parse(json).unwrap();
        assert_eq!(estimate.complexity_class, "Ω(n²)");
        assert_eq!(estimate.recommended_subtasks, 7);
    }

    #[test]
    fn test_invalid_complexity_class() {
        let json = r#"{
            "complexity_class": "O(n)",
            "reasoning": "Test",
            "recommended_subtasks": 3,
            "confidence": 0.8
        }"#;

        let result = ResponseParser::parse(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidComplexityClass(_)
        ));
    }

    #[test]
    fn test_invalid_subtask_count_zero() {
        let json = r#"{
            "complexity_class": "Ω(n)",
            "reasoning": "Test",
            "recommended_subtasks": 0,
            "confidence": 0.8
        }"#;

        let result = ResponseParser::parse(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidSubtaskCount(0)
        ));
    }

    #[test]
    fn test_invalid_subtask_count_too_high() {
        let json = r#"{
            "complexity_class": "Ω(n)",
            "reasoning": "Test",
            "recommended_subtasks": 25,
            "confidence": 0.8
        }"#;

        let result = ResponseParser::parse(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidSubtaskCount(25)
        ));
    }

    #[test]
    fn test_invalid_confidence_negative() {
        let json = r#"{
            "complexity_class": "Ω(n)",
            "reasoning": "Test",
            "recommended_subtasks": 3,
            "confidence": -0.1
        }"#;

        let result = ResponseParser::parse(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidConfidence(_)
        ));
    }

    #[test]
    fn test_invalid_confidence_too_high() {
        let json = r#"{
            "complexity_class": "Ω(n)",
            "reasoning": "Test",
            "recommended_subtasks": 3,
            "confidence": 1.5
        }"#;

        let result = ResponseParser::parse(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidConfidence(_)
        ));
    }

    #[test]
    fn test_all_complexity_classes() {
        let classes = vec![
            ("Ω(1)", ComplexityClass::Constant),
            ("Ω(log n)", ComplexityClass::Logarithmic),
            ("Ω(n)", ComplexityClass::Linear),
            ("Ω(n log n)", ComplexityClass::Linearithmic),
            ("Ω(n²)", ComplexityClass::Quadratic),
            ("Ω(2^n)", ComplexityClass::Exponential),
        ];

        for (class_str, expected) in classes {
            let json = format!(
                r#"{{
                "complexity_class": "{}",
                "reasoning": "Test",
                "recommended_subtasks": 3,
                "confidence": 0.8
            }}"#,
                class_str
            );

            let estimate = ResponseParser::parse(&json).unwrap();
            assert_eq!(estimate.to_complexity_class().unwrap(), expected);
        }
    }

    #[test]
    fn test_extract_json_plain() {
        let text = r#"{"key": "value"}"#;
        let result = ResponseParser::extract_json(text);
        assert_eq!(result, r#"{"key": "value"}"#);
    }

    #[test]
    fn test_extract_json_with_whitespace() {
        let text = r#"

        {"key": "value"}

        "#;
        let result = ResponseParser::extract_json(text);
        assert_eq!(result, r#"{"key": "value"}"#);
    }

    #[test]
    fn test_extract_json_markdown_block() {
        let text = r#"```json
{"key": "value"}
```"#;
        let result = ResponseParser::extract_json(text);
        assert_eq!(result, r#"{"key": "value"}"#);
    }
}
