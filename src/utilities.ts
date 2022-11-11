import { MD5 } from "crypto-js";
import { User } from "@/data/user";
import { Queue } from "@/data/queue";

import { Theme } from "@/data/settings";

import ColorHash from "color-hash";

export function getGravatarUrl(user: User): string {
  const email = user.email;
  const colorHash = new ColorHash({ lightness: 0.6, saturation: [0.3, 0.45, 0.6] });
  const color = colorHash.hex(user.fullName).substring(1);
  return `https://www.gravatar.com/avatar/${MD5(
    email.trim().toLowerCase(),
  ).toString()}?d=https%3A%2F%2Fui-avatars.com%2Fapi%2F/${encodeURIComponent(
    user.fullName.replace(" ", "+"),
  )}/128/${color}`;
}

export function userInQueue(user: User, queue: Queue): boolean {
  return queue.members.some(
    ({ fullName, username, email }) => user.fullName === fullName && user.username === username && user.email === email,
  );
}
export function getToastImage(queue: Queue): string | undefined {
  const queueName = queue.name;
  const regex = /#(\w+)\b(?!#)/;
  const hashtag = regex.exec(queueName)?.[1];
  if (!hashtag) {
    return undefined;
  }
  return `https://softwire.ontoast.io/hashtags/image/${hashtag}`;
}

import win98Css from "@/css/themes/98/98.scss?inline";
import classicQ3Css from "@/css/themes/default_theme.scss?inline";

export const themeCssMap: Record<Theme, string> = {
  Win98: win98Css,
  ClassicQ3: classicQ3Css,
  Modern: "",
} as const;
