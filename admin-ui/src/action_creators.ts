import { FreeDSL, dispatch, effect, Do } from "redux-free-flow";

import { Result, AuthData, AuthSuccess, GenericError } from "./interface";
import { Action, SyncAction } from "./actions";
import { signin as signinApi, signout as signoutApi, whoami } from "./api";
import { uid, match } from "./utils";

export function checkToken(): FreeDSL<void> {
  return _auth(effect(whoami), true);
}

export function signin(data: AuthData): FreeDSL<void> {
  return _auth(effect(signinApi(data)), false);
}

export function signout(): FreeDSL<void> {
  return effect(signoutApi).then(() => dispatch({ type: "LOGOUT" }));
}

function _auth(
  eff: FreeDSL<Result<AuthSuccess, GenericError>>,
  blocking: boolean
): FreeDSL<void> {
  const [request, response] = apiActions("signin");

  return Do(function*() {
    yield dispatch(request({ blocking }));
    const result: Result<AuthSuccess, GenericError> = yield eff;

    yield match(result, {
      Ok(res: AuthSuccess) {
        return dispatch(response({ type: "LOGIN_SUCCESS", payload: res }));
      },
      Err(err: GenericError) {
        return dispatch(response({ type: "LOGIN_ERROR", payload: err }));
      }
    });
  });
}

function apiActions(
  requestToken: string
): [(opt: { blocking: boolean }) => Action, (action: SyncAction) => Action] {
  const requestId = uid();

  function req({ blocking }: { blocking: boolean }): Action {
    return {
      type: "REQUEST",
      payload: {
        requestToken,
        requestId,
        blocking
      }
    };
  }
  function res(action: SyncAction): Action {
    return {
      type: "RESPONSE",
      payload: {
        requestToken,
        requestId,
        action
      }
    };
  }

  return [req, res];
}
