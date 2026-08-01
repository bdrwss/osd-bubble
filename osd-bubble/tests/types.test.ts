import { describe, it, expect } from "vitest";
import { DEFAULT_CUSTOM_STYLE, type CustomStyleParams } from "$lib/types";

describe("CustomStyleParams type and defaults", () => {
  it("DEFAULT_CUSTOM_STYLE has all required fields", () => {
    const defaults: CustomStyleParams = DEFAULT_CUSTOM_STYLE;
    expect(defaults).toHaveProperty("bg_color");
    expect(defaults).toHaveProperty("bg_opacity");
    expect(defaults).toHaveProperty("text_color");
    expect(defaults).toHaveProperty("border_color");
    expect(defaults).toHaveProperty("border_width");
    expect(defaults).toHaveProperty("radius");
  });

  it("DEFAULT_CUSTOM_STYLE values are sensible", () => {
    const d = DEFAULT_CUSTOM_STYLE;
    expect(typeof d.bg_color).toBe("string");
    expect(d.bg_color).toMatch(/^#[0-9a-fA-F]{6}$/);
    expect(typeof d.bg_opacity).toBe("number");
    expect(d.bg_opacity).toBeGreaterThanOrEqual(0);
    expect(d.bg_opacity).toBeLessThanOrEqual(1);
    expect(typeof d.text_color).toBe("string");
    expect(d.text_color).toMatch(/^#[0-9a-fA-F]{6}$/);
    expect(typeof d.border_color).toBe("string");
    expect(typeof d.border_width).toBe("number");
    expect(d.border_width).toBeGreaterThanOrEqual(0);
    expect(typeof d.radius).toBe("number");
    expect(d.radius).toBeGreaterThanOrEqual(0);
  });

  it("DEFAULT_CUSTOM_STYLE can be spread to create independent copy", () => {
    const copy = { ...DEFAULT_CUSTOM_STYLE };
    copy.bg_color = "#ff0000";
    expect(DEFAULT_CUSTOM_STYLE.bg_color).not.toBe("#ff0000");
  });
});
