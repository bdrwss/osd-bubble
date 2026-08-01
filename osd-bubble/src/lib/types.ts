export interface CustomStyleParams {
  bg_color: string;
  bg_opacity: number;
  text_color: string;
  border_color: string;
  border_width: number;
  radius: number;
  shadow_color: string;
}

export const DEFAULT_CUSTOM_STYLE: CustomStyleParams = {
  bg_color: "#000000",
  bg_opacity: 0.7,
  text_color: "#ffffff",
  border_color: "#000000",
  border_width: 0,
  radius: 8,
  shadow_color: "#000000",
};

export const BUBBLE_STYLES = [
  { id: "default", name: "默认样式" },
  { id: "3d_key", name: "3D 实体" },
  { id: "cartoon", name: "卡通泡泡" },
  { id: "retro_terminal", name: "极客终端" },
  { id: "custom", name: "自定义" },
] as const;
