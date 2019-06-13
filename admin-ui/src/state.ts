import { Bookmark } from "./interface";

export type State = AnnoState | AdminState<{ bookmark: Bookmark }>;

export const EMPTY_BOOKMARK: Bookmark = {
  title: "",
  url: "",
  body: "",
  tags: []
};

interface AnnoState {
  tag: "annoymous";
  value: null;
}

interface AdminState<S> {
  tag: "admin";
  value: S;
}
