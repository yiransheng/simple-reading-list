import {read, end, FreeDSL, dispatch, effect, Do} from 'redux-free-flow';

import {
  Result,
  AuthData,
  AuthSuccess,
  GenericError,
  Bookmark,
} from './interface';
import {State} from './reducers';
import {Action, SyncAction} from './actions';
import {
  createBookmark as createBookmarkApi,
  signin as signinApi,
  signout as signoutApi,
  whoami,
} from './api';
import {uid, match} from './utils';
import {selectBookmark, selectIsLoading} from './selectors';

export function checkToken(): FreeDSL<State, Action, void> {
  return _auth(effect(whoami), true);
}

export function signin(data: AuthData): FreeDSL<State, Action, void> {
  return _auth(effect(signinApi(data)), false);
}

export function signout(): FreeDSL<unknown, Action, void> {
  return effect(signoutApi).andThen(() => dispatch({type: 'LOGOUT'}));
}

export function editBookmark(payload: Bookmark): SyncAction {
  return {
    type: 'EDIT_BOOKMARK',
    payload,
  };
}

export function createBookmark(): FreeDSL<State, Action, void> {
  const [request, response] = apiActions("create_bookmark");

  return Do(function*() {
    const bookmark: ReturnType<typeof selectBookmark> = yield read(selectBookmark);
    const loading: boolean = yield read(selectIsLoading);

    let data;
    if (!loading && bookmark.tag === "Some") {
      data = bookmark.value;

      yield dispatch(request({ blocking: true }));
      const result: Result<void, GenericError> = yield effect(
        createBookmarkApi(data)
      );

      yield match(result, {
        Ok(_: void) {
          return dispatch(
            response({
              type: "BOOKMARK_CREATED",
              payload: {
                timestamp: new Date()
              }
            })
          );
        },
        Err(err: GenericError) {
          return dispatch(
            response({
              type: "BOOKMARK_CREATE_FAILURE",
              payload: { ...err, timestamp: new Date() }
            })
          );
        }
      });
    } else {
      yield end;
    }
  });
}

function _auth(
  eff: FreeDSL<State, Action, Result<AuthSuccess, GenericError>>,
  blocking: boolean,
): FreeDSL<State, Action, void> {
  const [request, response] = apiActions('signin');

  return Do(function*() {
    yield dispatch(request({blocking}));
    const result: Result<AuthSuccess, GenericError> = yield eff;

    yield match(result, {
      Ok(res: AuthSuccess) {
        return dispatch(response({type: 'LOGIN_SUCCESS', payload: res}));
      },
      Err(err: GenericError) {
        return dispatch(response({type: 'LOGIN_ERROR', payload: err}));
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
