import {Bookmark, AdminUser, GenericError} from './interface';

export type State = UnkState | AnnoState | AdminState<AdminInnerState>;

interface AdminInnerState {
  bookmark: Bookmark;
  user: AdminUser;
}

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
