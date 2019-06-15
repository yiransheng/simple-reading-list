import {Bookmark, AdminUser, GenericError} from './interface';

export type State = UnkState | AnnoState | AdminState<AdminInnerState>;

export type AppStatus = StatusNone | StatusOk | StatusErr;

export const DEFAULT_APP_STATUS: AppStatus = {tag: 'nothing', value: null};

interface AdminInnerState {
  bookmark: Bookmark;
  user: AdminUser;
  status: AppStatus;
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

interface StatusNone {
  tag: 'nothing';
  value: null;
}

interface StatusOk {
  tag: 'ok';
  value: {
    dismissWhen: Date;
    message: string;
  };
}

interface StatusErr {
  tag: 'err';
  value: {
    dismissWhen: Date;
    message: string;
  };
}
