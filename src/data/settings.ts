export const themes = ["Win98", "ClassicQ3", "Modern"] as const;
export type Theme = typeof themes[number];

export interface Settings {
  fullName: string;
  username: string;
  email: string;
  groups: [string];
  theme: Theme;
}
