/**
 * Theme System — Dark/Light theme support for ONEVOX TUI.
 *
 * Default: Dark theme (Vercel-inspired).
 * Toggle: Press 't' to switch between dark and light.
 * Persist: Saves to config.toml [ui] section.
 */

export type ThemeName = "dark" | "light"

export interface Theme {
  name: ThemeName
  colors: {
    // Backgrounds
    bg: string           // Main background
    surface: string      // Cards, elevated surfaces
    hover: string        // Hover states
    selected: string     // Selected/focused states
    border: string       // Dividers, borders
    statusBar: string    // Status bar background
    
    // Text
    textPrimary: string    // Headers, primary text
    textSecondary: string  // Labels, secondary text
    textMuted: string      // Hints, disabled text
    textInverse: string    // Text on dark backgrounds
    
    // Accents (still B&W)
    accent: string         // Primary accent (black in light, white in dark)
    accentMuted: string    // Muted accent
    
    // States
    active: string         // Active state (toggle ON)
    inactive: string       // Inactive state (toggle OFF)
    indicator: string      // Focus indicator (▶)
  }
}

export const DARK_THEME: Theme = {
  name: "dark",
  colors: {
    // Vercel dark theme
    bg: "#000000",              // Pure black
    surface: "#111111",         // Slightly lighter black
    hover: "#1A1A1A",          // Hover state
    selected: "#222222",        // Selected/focused
    border: "#333333",          // Subtle borders
    statusBar: "#0A0A0A",      // Darker status bar
    
    textPrimary: "#FFFFFF",     // Pure white text
    textSecondary: "#888888",   // Gray labels
    textMuted: "#666666",       // Muted hints
    textInverse: "#000000",     // Black text on white bg
    
    accent: "#FFFFFF",          // White accent
    accentMuted: "#666666",     // Gray accent
    
    active: "#FFFFFF",          // White for ON
    inactive: "#444444",        // Dark gray for OFF
    indicator: "#FFFFFF",       // White indicator
  }
}

export const LIGHT_THEME: Theme = {
  name: "light",
  colors: {
    // Vercel light theme (current)
    bg: "#FFFFFF",
    surface: "#FAFAFA",
    hover: "#F5F5F5",
    selected: "#F0F0F0",
    border: "#EAEAEA",
    statusBar: "#FAFAFA",
    
    textPrimary: "#000000",
    textSecondary: "#666666",
    textMuted: "#999999",
    textInverse: "#FFFFFF",
    
    accent: "#000000",
    accentMuted: "#999999",
    
    active: "#000000",
    inactive: "#CCCCCC",
    indicator: "#000000",
  }
}

export function getTheme(name: ThemeName): Theme {
  return name === "dark" ? DARK_THEME : LIGHT_THEME
}
