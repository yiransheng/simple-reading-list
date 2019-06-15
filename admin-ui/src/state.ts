import {Bookmark} from './interface';

export type State = UnkState | AnnoState | AdminState<{bookmark: Bookmark}>;

export const EMPTY_BOOKMARK: Bookmark = {
  title: '',
  url: '',
  body: '',
  tags: [],
};

interface UnkState {
  tag: 'unknown';
  value: null;
}

interface AnnoState {
  tag: 'annoymous';
  value: null;
}

interface AdminState<S> {
  tag: 'admin';
  value: S;
}
