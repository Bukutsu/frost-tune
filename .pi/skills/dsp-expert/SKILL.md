---
name: dsp-expert
description: Expert in Biquad filter calculations and audio DSP.
---

# DSP Validation Protocol

Use this skill when modifying `src/hardware/dsp.rs` or calculating filter coefficients.

## Rules & Guidelines
1. **Verify Formulas**: Always cross-reference Biquad formulas with Robert Bristow-Johnson's (RBJ) Audio EQ Cookbook.
2. **Stability Check**: Ensure the filter remains stable (all poles must be inside the unit circle).
3. **Verification Step**: Before applying a coefficient change to the hardware, the agent should propose a Python snippet using `scipy.signal.freqz` to plot the expected frequency and phase response.
4. **Precision**: Use `f32` (32-bit float) precision consistently, and be mindful of quantization errors when targeting fixed-point hardware if applicable.

## When to Activate
- Creating new filter types (Low-pass, High-pass, Shelving, Peaking).
- Implementing Gain/Q to Coefficient conversion logic.
- Debugging audio artifacts or unexpected frequency responses.
