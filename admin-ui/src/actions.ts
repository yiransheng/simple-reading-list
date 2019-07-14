import {Action} from 'redux';
import {Bookmark, AuthSuccess, GenericError} from './interface';

export type Action = RequestAction | ResponseAction | SyncAction;

export type SyncAction =
  | LoginSuccessAction
  | LoginErrorAction
  | LogoutAction
  | EditBookmarkAction
  | CreateErrorAction
  | CreateSuccessAction;

export interface RequestAction {
  type: 'REQUEST';
  payload: {
    requestToken: string;
    requestId: number;
    blocking: boolean;
  };
}
export interface ResponseAction {
  type: 'RESPONSE';
  payload: {
    requestToken: string;
    requestId: number;
    action: SyncAction;
  };
}

export interface LoginSuccessAction {
  type: 'LOGIN_SUCCESS';
  payload: AuthSuccess;
}
export interface LoginErrorAction {
  type: 'LOGIN_ERROR';
  payload: GenericError;
}
export interface LogoutAction {
  type: 'LOGOUT';
}
export interface EditBookmarkAction {
  type: 'EDIT_BOOKMARK';
  payload: Bookmark;
}
export interface CreateSuccessAction {
  type: 'BOOKMARK_CREATED';
  payload: {
    timestamp: Date;
  };
}
export interface CreateErrorAction {
  type: 'BOOKMARK_CREATE_FAILURE';
  payload: {
    timestamp: Date;
  } & GenericError;
}
