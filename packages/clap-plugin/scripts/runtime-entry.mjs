// Strudel Runtime Entry Point for Plugin
// This file exports a minimal API for the CLAP/VST3 plugin to evaluate patterns
// Copyright (C) 2026 Strudel Contributors
// Licensed under AGPL-3.0-or-later

import { Pattern, State, TimeSpan, Hap, Fraction } from '@strudel/core';
import { transpiler } from '@strudel/transpiler';
import * as strudel from '@strudel/core';
import * as mini from '@strudel/mini';

// Make all Strudel functions available globally for user code
Object.assign(globalThis, strudel);
Object.assign(globalThis, mini);

// Fraction is the wrapper function, get the actual constructor
const FractionConstructor = Fraction._original || Fraction;

// CRITICAL: Register mini notation parser so s('bd sd') works
// This makes all string arguments to pattern functions parse as mini notation
mini.miniAllStrings();

// Global state
let currentPattern = null;
let currentBPM = 120;
let evaluationError = null;

/**
 * Evaluate Strudel code and store the resulting pattern
 * @param {string} code - User code to evaluate
 * @returns {object} - Result with success status and optional error
 */
globalThis.evaluateCode = async function(code) {
  try {
    evaluationError = null;

    // Transpile the code
    const transpiled = transpiler(code);

    // Evaluate using the same pattern as evaluate.mjs
    // Wrap in block expression and async IIFE
    const wrappedCode = `(async ()=>{${transpiled.output}})()`;
    const body = `"use strict";return (${wrappedCode})`;
    const evaluator = Function(body);

    // Execute and await the result
    const result = await evaluator();

    // Store the pattern globally
    globalThis.currentPattern = result;

    if (result && result.query && typeof result.query === 'function') {
      currentPattern = result;
      return { success: true };
    } else {
      throw new Error('Code did not produce a valid Pattern');
    }
  } catch (error) {
    evaluationError = error.message;
    currentPattern = null;
    globalThis.currentPattern = null;
    return {
      success: false,
      error: error.message,
      stack: error.stack
    };
  }
};

/**
 * Query the current pattern for events in a time range
 * @param {number} startNum - Start time numerator
 * @param {number} startDen - Start time denominator
 * @param {number} endNum - End time numerator
 * @param {number} endDen - End time denominator
 * @returns {Array} - Array of events (haps) serialized for Rust
 */
globalThis.queryPattern = function(startNum, startDen, endNum, endDen) {
  if (!currentPattern) {
    return [];
  }

  try {
    // Create fractions for start and end times
    // Use the constructor, not the wrapper
    const startFrac = new FractionConstructor(startNum, startDen);
    const endFrac = new FractionConstructor(endNum, endDen);

    // Create TimeSpan and State for querying
    const span = new TimeSpan(startFrac, endFrac);
    const state = new State(span);

    // Query the pattern
    const haps = currentPattern.query(state);

    // Serialize haps for Rust consumption
    return haps.map(hap => {
      // Determine event type based on value
      const eventType = hap.value.s ? 'audio' : 'midi';

      // Extract onset and duration as fractions
      const onset = hap.whole?.begin || hap.part.begin;
      const duration = hap.duration || new FractionConstructor(0);

      return {
        onset: [Number(onset.n), Number(onset.d)],
        duration: [Number(duration.n), Number(duration.d)],
        event_type: eventType,
        params: hap.value
      };
    });
  } catch (error) {
    console.error('Error querying pattern:', error.message, error.stack);
    return [];
  }
};

/**
 * Set the tempo for timing calculations
 * @param {number} bpm - Beats per minute
 */
globalThis.setTempo = function(bpm) {
  currentBPM = bpm;
};

/**
 * Get the current BPM
 * @returns {number} - Current BPM
 */
globalThis.getTempo = function() {
  return currentBPM;
};

/**
 * Get the last evaluation error, if any
 * @returns {string|null} - Error message or null
 */
globalThis.getLastError = function() {
  return evaluationError;
};

/**
 * Clear the current pattern
 */
globalThis.clearPattern = function() {
  currentPattern = null;
  evaluationError = null;
};

/**
 * Test pattern for verification
 */
globalThis.testPattern = function() {
  const code = 's("bd sd hh sd").fast(2)';
  return evaluateCode(code);
};

console.log('Strudel runtime initialized');
