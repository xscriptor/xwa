```markdown
# Design System Strategy: The Cyber-Audit Protocol

## 1. Overview & Creative North Star: "The Synthetic Architect"
This design system moves beyond the cliché "hacker" tropes of the 90s to create a high-end, brutalist digital environment. Our Creative North Star is **"The Synthetic Architect."** The aesthetic is intentional, high-contrast, and information-dense, mimicking a high-clearance tactical interface. 

We break the "template" look by rejecting soft rounded corners and standard padding. Instead, we embrace **Hard-Edge Asymmetry**. Layouts should feel like a modular HUD (Heads-Up Display), where data isn't just displayed—it is "intercepted." We use overlapping scan lines, offset "glitch" containers, and varying information densities to create a sense of urgent, professional technicality.

---

## 2. Colors & Surface Architecture
The palette is rooted in absolute voids and high-frequency luminescent accents.

### Color Strategy
- **Primary (`#39FF14` / `primary_container`):** The "Execute" signal. Used for successful scans and critical active states.
- **Secondary (`#00F3FF` / `secondary_container`):** The "Analysis" signal. Used for data visualization and navigational wayfinding.
- **Tertiary (`#FFB000` / `tertiary_fixed`):** The "Alert" signal. Reserved strictly for warnings, vulnerabilities, and high-risk audit findings.

### The "No-Line" Rule
Standard 1px solid borders are strictly prohibited for sectioning. Structural boundaries must be defined through **Background Color Shifts**. For example:
- A main dashboard area uses `surface`.
- A nested data panel uses `surface_container_low`.
- A critical "Deep Scan" sidebar uses `surface_container_highest`.
By using these tonal shifts, the UI feels like a solid, machined object rather than a collection of boxes.

### Surface Hierarchy & Nesting
Treat the UI as a series of physical layers. 
- **Layer 0 (Background):** `surface_dim` (#131313).
- **Layer 1 (Modules):** `surface_container`.
- **Layer 2 (Active Highlighting):** `surface_bright`.
When nesting elements, use the `surface_container_lowest` for inner data cells to create a "recessed" terminal feel.

### The "Glass & Gradient" Rule
To add "soul," use **Glassmorphism** for floating HUD elements. Apply `surface_container_low` at 60% opacity with a 20px `backdrop-blur`. Main CTAs should not be flat; apply a subtle linear gradient from `primary` to `primary_container` to simulate a glowing cathode-ray tube (CRT) effect.

---

## 3. Typography: Technical Authority
We pair high-precision monospace with aggressive, sharp sans-serifs to distinguish between *Analysis* and *Action*.

- **Display & Headlines (`Space Grotesk`):** Used for high-level status and section headers. The sharp terminals and tight apertures of Space Grotesk mirror the "hard-edge" philosophy.
- **Body & Data (`Inter` / Monospace for Results):** While `Inter` handles standard UI labels, all audit results, IP addresses, and code snippets must use a Monospace font.
- **Hierarchy via Scale:** Use extreme contrast. A `display-lg` (3.5rem) title might sit directly next to a `label-sm` (0.68rem) timestamp to create an editorial, "technical blueprint" aesthetic.

---

## 4. Elevation & Depth: Tonal Layering
In this system, "Up" is defined by light and transparency, not by shadows.

- **The Layering Principle:** Avoid traditional drop shadows. Instead, stack your tokens. An "active" card is recognized because it shifts from `surface_container_low` to `surface_container_high`.
- **Ambient Glows:** When an element must "float," use an **Ambient Glow** instead of a shadow. Apply a blur to a low-opacity version of the `primary` (green) or `secondary` (blue) token. This simulates light emission from the screen itself.
- **The "Ghost Border":** For high-density data grids where separation is vital, use the `outline_variant` token at 15% opacity. It should feel like a faint grid line on a blueprint, not a structural wall.
- **Glitch & Scan Lines:** Use a repeating linear-gradient overlay on `surface_bright` elements to create "Scanning Lines." This adds a tactile, hardware-inspired depth to the interface.

---

## 5. Components & Primitive Styling

### Buttons
- **Primary:** `0px` border radius. Solid `primary_container`. Text in `on_primary_fixed`. On hover, add a 2px "offset" stroke in `primary`.
- **Secondary:** Transparent background, `primary` ghost border (20% opacity), text in `primary`.
- **Tertiary:** Text only, monospace, all caps, with a `_` prefix (e.g., `_RUN_DIAGNOSTICS`).

### Cards & Data Modules
- **Forbid Dividers:** Use vertical white space (`spacing.8`) or tonal shifts (`surface_container_low`) to separate scan results.
- **Terminal Headers:** Every card should have a small "header bar" using `surface_container_highest` containing metadata (e.g., `STATUS: 200`, `LOC: US-EAST`).

### Input Fields
- Rectangular, `0px` radius. 
- **Active State:** The border glows with `secondary_fixed_dim`. 
- **Error State:** The background shifts to a subtle `error_container` (10% opacity) with `tertiary` text accents.

### Custom Components: "The Scan-Bar"
A progress component unique to this system. Instead of a smooth bar, use a series of segmented blocks that fill from left to right, flickering slightly using a "glitch" CSS animation when data is being processed.

---

## 6. Do's and Don'ts

### Do:
- **Use Monospace for Numbers:** Always use monospace for percentages, counts, and IDs to ensure vertical alignment in data-heavy views.
- **Embrace Information Density:** This tool is for experts. Don't be afraid of "crowded" screens; use clear typography scales to guide the eye.
- **Intentional Asymmetry:** Offset your headers or use "cut corners" (via CSS clip-path) to make containers feel custom-built.

### Don't:
- **No Rounded Corners:** `0px` is the law. Any radius softens the "hacker" edge and ruins the Synthetic Architect aesthetic.
- **No Standard Grey Shadows:** If it doesn't glow, it doesn't lift.
- **No Generic Icons:** Use sharp, geometric icons. Avoid "friendly" or "bubbly" iconography. If a "Home" icon is needed, it should look like a structural floor plan, not a cottage.
- **No Divider Lines:** If you feel the need for a line, use a spacing increment of `spacing.px` with a `surface_container_high` background shift instead.

---

## 7. Spacing Scale
Our spacing is rigid and mathematical. 
- Use **Small Gaps** (`0.2rem` to `0.5rem`) for related data points.
- Use **Large Voids** (`3.5rem` to `5.5rem`) to separate major functional blocks. 
- **Alignment:** Always align text to the hard left edge of its container to maintain the "Terminal" look. Avoid center-aligned text in data modules.```