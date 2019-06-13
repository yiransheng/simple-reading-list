import {Action} from 'redux';
import {AuthSuccess, AuthError} from './interface';

export type Action = RequestAction | ResponseAction | SyncAction;

export type SyncAction = LoginSuccessAction | LoginErrorAction;

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
  payload: AuthError;
}
