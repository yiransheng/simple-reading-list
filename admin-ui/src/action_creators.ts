import {read, end, FreeDSL, dispatch, effect, Do} from 'redux-free-flow';

import {
  Result,
  AuthData,
  AuthSuccess,
  GenericError,
  Option,
  Bookmark,
} from './interface';
import {Action, SyncAction} from './actions';
import {
  createBookmark as createBookmarkApi,
  signin as signinApi,
  signout as signoutApi,
  whoami,
} from './api';
import {uid, match} from './utils';
import {selectBookmark, selectIsLoading} from './selectors';
import {ReduxDSL} from './store';

export function checkToken(): ReduxDSL<void> {
  return _auth(effect(whoami), true);
}

export function signin(data: AuthData): ReduxDSL<void> {
  return _auth(effect(signinApi(data)), false);
}

export function signout(): ReduxDSL<void> {
  return effect(signoutApi).then(() => dispatch({type: 'LOGOUT'})).phantom();
}

export function editBookmark(payload: Bookmark): SyncAction {
  return {
    type: 'EDIT_BOOKMARK',
    payload,
  };
}

export function createBookmark(): ReduxDSL<void> {
  const [request, response] = apiActions('create_bookmark');

  return Do(function*() {
    const bookmark: Option<Bookmark> = yield read(selectBookmark);
    const loading: boolean = yield read(selectIsLoading);

    let data;
    if (!loading && bookmark.tag === 'Some') {
      data = bookmark.value;
    } else {
      return end.phantom();
    }

    yield dispatch(request({blocking: true}));
    const result: Result<void, GenericError> = yield effect(
      createBookmarkApi(data),
    );

    yield match(result, {
      Ok(_: void) {
        return dispatch(
          response({
            type: 'BOOKMARK_CREATED',
            payload: {
              timestamp: new Date(),
            },
          }),
        ).phantom();
      },
      Err(err: GenericError) {
        return dispatch(
          response({
            type: 'BOOKMARK_CREATE_FAILURE',
            payload: {...err, timestamp: new Date()},
          }),
        ).phantom();
      },
    });
  });
}

function _auth(
  eff: ReduxDSL<Result<AuthSuccess, GenericError>>,
  blocking: boolean,
): ReduxDSL<void> {
  const [request, response] = apiActions('signin');

  return Do(function*() {
    yield dispatch(request({blocking}));
    const result: Result<AuthSuccess, GenericError> = yield eff;

    yield match(result, {
      Ok(res: AuthSuccess) {
        return dispatch(response({type: 'LOGIN_SUCCESS', payload: res})).phantom();
      },
      Err(err: GenericError) {
        return dispatch(response({type: 'LOGIN_ERROR', payload: err})).phantom();
      },
    });
  });
}

function apiActions(
  requestToken: string,
): [(opt: {blocking: boolean}) => Action, (action: SyncAction) => Action] {
  const requestId = uid();

  function req({blocking}: {blocking: boolean}): Action {
    return {
      type: 'REQUEST',
      payload: {
        requestToken,
        requestId,
        blocking,
      },
    };
  }
  function res(action: SyncAction): Action {
    return {
      type: 'RESPONSE',
      payload: {
        requestToken,
        requestId,
        action,
      },
    };
  }

  return [req, res];
}
