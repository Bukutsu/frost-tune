---
name: material-ui-auditor
description: Ensures Iced UI components adhere to Material Design 3.
---

# Material 3 / Iced Guidelines

Use this skill when modifying `src/ui/` or updating the application's visual style.

## Layout & Spacing
1. **8px Grid**: All margins, padding, and spacing MUST be multiples of 8px (e.g., 8, 16, 24, 32).
2. **Consistency**: Maintain consistent alignment across different views. Use standard Material 3 component padding.

## Theme & Color
1. **Tokyo Night Mapping**: Map Material 3 roles (Surface, Primary, On-Surface, Error) strictly to the project's **Tokyo Night** palette tokens.
2. **Elevation**: Use subtle color shifts to represent elevation (Surface 0, 1, 2) rather than heavy shadows.

## Technical Constraints
1. **Iced 0.14 API**: Only use API calls compatible with Iced 0.14. Avoid deprecated builder patterns.
2. **Responsiveness**: Verify that all widgets are responsive and handle window resizing without clipping or awkward stretching.

## Verification
Before finalizing UI changes, describe the layout in terms of "spacing units" (e.g., "Added 2 units (16px) of padding to the container").
