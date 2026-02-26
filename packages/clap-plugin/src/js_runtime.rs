// JavaScript Runtime Integration for Strudel Plugin
// Copyright (C) 2026 Strudel Contributors
// Licensed under AGPL-3.0-or-later

use deno_core::error::AnyError;
use deno_core::{JsRuntime, RuntimeOptions, serde_json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::runtime::Runtime as TokioRuntime;

/// Strudel event produced by pattern queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrudelEvent {
    /// Event onset time as a fraction (numerator, denominator)
    pub onset: (i64, i64),
    /// Event duration as a fraction (numerator, denominator)
    pub duration: (i64, i64),
    /// Event type: "audio" or "midi"
    pub event_type: String,
    /// Full event parameters (Strudel hap.value)
    pub params: serde_json::Value,
}

/// Result of evaluating Strudel code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub success: bool,
    pub error: Option<String>,
    pub stack: Option<String>,
}

/// Strudel JavaScript runtime wrapper
pub struct StrudelRuntime {
    runtime: Arc<RwLock<JsRuntime>>,
    tokio_runtime: Arc<TokioRuntime>,
}

impl StrudelRuntime {
    /// Create a new Strudel runtime with embedded JavaScript bundle
    pub fn new() -> Result<Self, AnyError> {
        // Embedded JavaScript bundle (generated at compile time)
        const STRUDEL_BUNDLE: &str = include_str!("../bundled/strudel-runtime.js");

        // Create Deno runtime with default options
        let mut runtime = JsRuntime::new(RuntimeOptions {
            ..Default::default()
        });

        // Inject browser API polyfills that Strudel needs
        let polyfills = r#"
            // Mock performance API
            globalThis.performance = {
                now: () => Date.now(),
                timeOrigin: Date.now()
            };

            // Mock window (some Strudel code checks for it)
            globalThis.window = globalThis;

            // Mock document
            globalThis.document = {
                createElement: () => ({}),
                getElementById: () => null,
                querySelector: () => null,
                querySelectorAll: () => [],
                addEventListener: () => {},
                removeEventListener: () => {}
            };

            // Mock console if not available
            if (typeof console === 'undefined') {
                globalThis.console = {
                    log: () => {},
                    warn: () => {},
                    error: () => {},
                    info: () => {},
                    debug: () => {}
                };
            }

            // Prevent SharedWorker errors
            globalThis.SharedWorker = class SharedWorker {
                constructor() {
                    console.warn('SharedWorker not available in plugin context');
                }
            };
        "#;

        runtime.execute_script("[polyfills]", polyfills)?;

        // Execute the bundled Strudel runtime
        runtime.execute_script("[strudel-runtime]", STRUDEL_BUNDLE)?;

        // Create Tokio runtime for async operations
        let tokio_runtime = TokioRuntime::new()?;

        Ok(Self {
            runtime: Arc::new(RwLock::new(runtime)),
            tokio_runtime: Arc::new(tokio_runtime),
        })
    }

    /// Set the pattern code to evaluate
    /// This transpiles and evaluates the code, storing the resulting pattern
    pub fn set_code(&self, code: &str) -> EvaluationResult {
        // Execute evaluation asynchronously
        let result = self.tokio_runtime.block_on(async {
            let mut runtime = self.runtime.write();

            // Call evaluateCode() function from the JavaScript runtime (returns a Promise)
            let eval_script = format!(
                "globalThis.evaluateCode({})",
                serde_json::to_string(code).unwrap()
            );

            // Execute the script (returns a Promise)
            match runtime.execute_script("[eval-code]", eval_script) {
                Ok(_) => {
                    // Run the event loop to resolve the promise
                    match runtime.run_event_loop(Default::default()).await {
                        Ok(_) => {
                            // Get the result by calling a sync wrapper
                            let get_result = "JSON.stringify({ success: currentPattern !== null, error: globalThis.getLastError() })";
                            match runtime.execute_script("[get-eval-result]", get_result) {
                                Ok(value_ref) => {
                                    let scope = &mut runtime.handle_scope();
                                    let value = value_ref.open(scope);
                                    let json_str = value.to_rust_string_lossy(scope);

                                    serde_json::from_str(&json_str).unwrap_or(EvaluationResult {
                                        success: false,
                                        error: Some("Failed to parse evaluation result".to_string()),
                                        stack: None,
                                    })
                                }
                                Err(e) => EvaluationResult {
                                    success: false,
                                    error: Some(e.to_string()),
                                    stack: None,
                                },
                            }
                        }
                        Err(e) => EvaluationResult {
                            success: false,
                            error: Some(format!("Event loop error: {}", e)),
                            stack: None,
                        },
                    }
                }
                Err(e) => EvaluationResult {
                    success: false,
                    error: Some(e.to_string()),
                    stack: None,
                },
            }
        });

        result
    }

    /// Query the pattern for events in a cycle range
    /// Returns events with onset times as fractions
    pub fn query_pattern(
        &self,
        start: (i64, i64),
        end: (i64, i64),
    ) -> Result<Vec<StrudelEvent>, AnyError> {
        let mut runtime = self.runtime.write();

        // Call queryPattern() function from JavaScript
        let query_script = format!(
            "JSON.stringify(globalThis.queryPattern({}, {}, {}, {}))",
            start.0, start.1, end.0, end.1
        );

        let value_ref = runtime.execute_script("[query-pattern]", query_script)?;
        let scope = &mut runtime.handle_scope();
        let value = value_ref.open(scope);
        let json_str = value.to_rust_string_lossy(scope);

        let events: Vec<StrudelEvent> = serde_json::from_str(&json_str)?;
        Ok(events)
    }

    /// Set the tempo (BPM) for timing calculations
    pub fn set_tempo(&self, bpm: f64) -> Result<(), AnyError> {
        let mut runtime = self.runtime.write();

        let tempo_script = format!("globalThis.setTempo({})", bpm);
        runtime.execute_script("[set-tempo]", tempo_script)?;

        Ok(())
    }

    /// Get the current tempo
    pub fn get_tempo(&self) -> Result<f64, AnyError> {
        let mut runtime = self.runtime.write();

        let value_ref = runtime.execute_script("[get-tempo]", "globalThis.getTempo()")?;
        let scope = &mut runtime.handle_scope();
        let value = value_ref.open(scope);

        Ok(value.number_value(scope).unwrap_or(120.0))
    }

    /// Get the last evaluation error, if any
    pub fn get_last_error(&self) -> Result<Option<String>, AnyError> {
        let mut runtime = self.runtime.write();

        let value_ref =
            runtime.execute_script("[get-error]", "globalThis.getLastError()")?;
        let scope = &mut runtime.handle_scope();
        let value = value_ref.open(scope);

        if value.is_null() || value.is_undefined() {
            Ok(None)
        } else {
            Ok(Some(value.to_rust_string_lossy(scope)))
        }
    }

    /// Clear the current pattern
    pub fn clear_pattern(&self) -> Result<(), AnyError> {
        let mut runtime = self.runtime.write();
        runtime.execute_script("[clear-pattern]", "globalThis.clearPattern()")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = StrudelRuntime::new();
        assert!(runtime.is_ok(), "Runtime should initialize successfully");
    }

    #[test]
    fn test_set_get_tempo() {
        let runtime = StrudelRuntime::new().unwrap();

        // Set tempo to 140 BPM
        runtime.set_tempo(140.0).unwrap();

        // Get tempo
        let tempo = runtime.get_tempo().unwrap();
        assert_eq!(tempo, 140.0, "Tempo should be 140 BPM");
    }

    #[test]
    fn test_evaluate_simple_pattern() {
        let runtime = StrudelRuntime::new().unwrap();

        // Evaluate a simple pattern
        let result = runtime.set_code("s('bd sd')");

        assert!(
            result.success,
            "Pattern evaluation should succeed. Error: {:?}",
            result.error
        );
        assert!(result.error.is_none(), "There should be no error");
    }

    #[test]
    fn test_evaluate_invalid_code() {
        let runtime = StrudelRuntime::new().unwrap();

        // Try to evaluate invalid code
        let result = runtime.set_code("this is not valid code");

        assert!(!result.success, "Invalid code should fail");
        assert!(result.error.is_some(), "There should be an error message");
    }

    #[test]
    fn test_query_pattern() {
        let runtime = StrudelRuntime::new().unwrap();

        // Set a pattern
        let result = runtime.set_code("s('bd sd hh sd')");
        assert!(result.success, "Pattern should evaluate successfully");

        // Query for events in first cycle
        let events = runtime.query_pattern((0, 1), (1, 1)).unwrap();

        assert!(!events.is_empty(), "Should produce at least one event");
        assert_eq!(events.len(), 4, "Should produce 4 events (bd sd hh sd)");

        // Check first event
        let first_event = &events[0];
        assert_eq!(first_event.event_type, "audio", "Should be audio event");
        assert_eq!(first_event.onset.0, 0, "First event should start at 0");
    }

    #[test]
    fn test_query_fast_pattern() {
        let runtime = StrudelRuntime::new().unwrap();

        // Test the example from the plan: s("bd sd hh sd").fast(2)
        let result = runtime.set_code("s('bd sd hh sd').fast(2)");
        assert!(result.success, "Pattern should evaluate successfully");

        // Query for events in first cycle
        let events = runtime.query_pattern((0, 1), (1, 1)).unwrap();

        assert_eq!(
            events.len(),
            8,
            "Should produce 8 events per cycle with fast(2)"
        );
    }

    #[test]
    fn test_clear_pattern() {
        let runtime = StrudelRuntime::new().unwrap();

        // Set a pattern
        runtime.set_code("s('bd')");

        // Clear the pattern
        runtime.clear_pattern().unwrap();

        // Query should return empty
        let events = runtime.query_pattern((0, 1), (1, 1)).unwrap();
        assert!(events.is_empty(), "Cleared pattern should produce no events");
    }
}
