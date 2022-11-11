import { User } from "@/data/user";
import { Message } from "@/data/message";

export interface Queue {
  id: number;
  name: string;
  status: "Open" | "Started" | "Closed";
  members: [User];
  messages: [Message];
  restrictToGroup: string;
}
