import { describe, it, expect } from "vitest";
import { mount } from "svelte";
import CustomStyleEditor from "$lib/components/CustomStyleEditor.svelte";
import { DEFAULT_CUSTOM_STYLE } from "$lib/types";

describe("CustomStyleEditor component", () => {
  it("mounts without errors with valid props", () => {
    const target = document.createElement("div");
    const customStyle = { ...DEFAULT_CUSTOM_STYLE };
    const onReset = () => {};

    const component = mount(CustomStyleEditor, {
      target,
      props: { customStyle, onReset },
    });

    expect(component).toBeDefined();
    expect(target.innerHTML).toContain("自定义样式");
    expect(target.innerHTML).toContain("背景色");
    expect(target.innerHTML).toContain("透明度");
    expect(target.innerHTML).toContain("文字色");
    expect(target.innerHTML).toContain("圆角");
    expect(target.innerHTML).toContain("边框");
  });

  it("renders reset button", () => {
    const target = document.createElement("div");
    const customStyle = { ...DEFAULT_CUSTOM_STYLE };
    const onReset = () => {};

    mount(CustomStyleEditor, {
      target,
      props: { customStyle, onReset },
    });

    const resetBtn = target.querySelector(".btn-reset");
    expect(resetBtn).not.toBeNull();
    expect(resetBtn!.textContent).toContain("重置");
  });

  it("renders all 6 controls in grid", () => {
    const target = document.createElement("div");
    const customStyle = { ...DEFAULT_CUSTOM_STYLE };
    const onReset = () => {};

    mount(CustomStyleEditor, {
      target,
      props: { customStyle, onReset },
    });

    const controls = target.querySelectorAll(".control");
    expect(controls.length).toBe(6);
  });
});
